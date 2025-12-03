use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use axum::extract::FromRequestParts;
use axum::http::HeaderValue;
use axum::{extract::Request, response::Response};
use derivative::Derivative;
use dioxus::server::ServerFnError;
use futures_util::future::BoxFuture;
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use tokio::sync::Mutex;
use tower::{Layer, Service};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::adapters::traits::{LoginSession, School};
use crate::server::state::ServerState;

/// A login process. Its lifecycle is:
/// - Lifecycle starts when the user access this website. The user gets a session ID now.
/// - Then the user selects a school API, and logins with username and password.
/// - After that, we associate the session ID with a [`Credentials`], storing that in DB.
/// - Now this session is done, and is removed from the HashMap in LoginProcessManager.
#[derive(Derivative)]
#[derivative(Debug)]
enum LoginProcessInner {
    Started {
        school_adapters: Arc<Mutex<HashMap<&'static str, Arc<dyn School>>>>,
    },
    SelectedSchool {
        school: Arc<dyn School>,
        session: Box<dyn LoginSession>,
    },
    Finished {
        school: Arc<dyn School>,
        cred_db_key: String,
    },
}

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct LoginProcess {
    inner: Arc<Mutex<LoginProcessInner>>,
}

impl LoginProcess {
    /// Start a new session
    pub fn start(school_adapters: Arc<Mutex<HashMap<&'static str, Arc<dyn School>>>>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LoginProcessInner::Started { school_adapters })),
        }
    }

    /// Set the school adapter for this session
    pub async fn select_school(&self, school_name: String) -> Result<()> {
        let mut inner = self.inner.lock().await;
        let LoginProcessInner::Started { school_adapters } = &*inner else {
            bail!("Cannot select school: Not in started state")
        };
        let school = school_adapters
            .lock()
            .await
            .get(school_name.as_str())
            .cloned()
            .context("No such school")?;

        let login_session = school
            .new_login_session()
            .await
            .context("Creating new login session for school")?;
        *inner = LoginProcessInner::SelectedSchool {
            school,
            session: login_session,
        };
        tracing::info!("Server side selected school: {school_name}");

        Ok(())
    }

    /// Get the captcha image content
    pub async fn get_captcha(&self) -> Result<DynamicImage> {
        let inner = self.inner.lock().await;
        let LoginProcessInner::SelectedSchool { session, .. } = &*inner else {
            bail!("Not in SelectedSchool when calling `get_captcha`. Session: {self:#?}");
        };

        Ok(session.get_captcha().clone())
    }

    pub async fn login(
        &self,
        username: String,
        password: String,
        captcha_answer: String,
    ) -> Result<String> {
        let mut inner = self.inner.lock().await;
        let LoginProcessInner::SelectedSchool { school, session } = &*inner else {
            bail!("Not in SelectedSchool when calling `login`: {inner:?}");
        };

        let cred = session.login(username, password, captcha_answer).await?;
        let cred_db_key = session.save_cred_to_db(cred).await?;

        *inner = LoginProcessInner::Finished {
            school: school.clone(),
            cred_db_key: cred_db_key.clone(),
        };

        Ok(cred_db_key)
    }

    pub async fn cred_db_key(&self) -> Option<String> {
        let inner = self.inner.lock().await;
        if let LoginProcessInner::Finished { cred_db_key, .. } = &*inner {
            Some(cred_db_key.clone())
        } else {
            None
        }
    }

    pub async fn selected_school_adapter_name(&self) -> Option<String> {
        let inner = self.inner.lock().await;

        match &*inner {
            LoginProcessInner::Started { .. } => None,
            LoginProcessInner::SelectedSchool { school, .. } => {
                Some(school.adapter_name().to_string())
            }
            LoginProcessInner::Finished { school, .. } => Some(school.adapter_name().to_string()),
        }
    }
}

impl<S: Sync> FromRequestParts<S> for LoginProcess {
    type Rejection = ServerFnError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<Self>()
            .expect("No LoginProcess found in extension. Is `LoginProcessManagerLayer` setup?")
            .clone())
    }
}

// === Layer ===
// Use when setting up routes

#[derive(Clone, Debug)]
pub struct LoginProcessManagerLayer {
    all_processes: Arc<std::sync::Mutex<HashMap<String, LoginProcess>>>,
}

impl Default for LoginProcessManagerLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoginProcessManagerLayer {
    pub fn new() -> Self {
        LoginProcessManagerLayer {
            all_processes: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl<S> Layer<S> for LoginProcessManagerLayer {
    type Service = LoginProcessManager<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoginProcessManager {
            inner,
            all_processes: self.all_processes.clone(),
        }
    }
}

// === Middleware ===

#[derive(Clone, Debug)]
pub struct LoginProcessManager<S> {
    inner: S,
    all_processes: Arc<std::sync::Mutex<HashMap<String, LoginProcess>>>,
}

impl<S> Service<Request> for LoginProcessManager<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        const COOKIE_KEY: &str = "session_id";

        // Incoming request
        let cookies = request
            .extensions()
            .get::<Cookies>()
            .expect("Cannot get cookies. Is `CookieManagerLayer` configured?");
        let session_id = cookies.get(COOKIE_KEY);
        let mut all_progresses = self.all_processes.lock().unwrap();

        let (set_cookie, process) = if let Some(session_id) = &session_id
            && let Some(progress) = all_progresses.get(session_id.value())
        {
            // Session found, insert into extensions
            (None, progress.clone())
        } else {
            // Session invalid or not found, create a new session and insert into extensions
            let server_state = request
                .extensions()
                .get::<ServerState>()
                .expect("ServerState not found in extensions");
            let school_adapters = server_state.school_adapters.clone();
            let new_session_id = Uuid::new_v4().to_string();
            let new_process = LoginProcess::start(school_adapters);

            if let Some(invalid_session) = session_id {
                // Session invalid
                let invalid_session = invalid_session.value();
                tracing::warn!(
                    "Got invalid session_id {invalid_session} from client, assigning new one {new_session_id}"
                );
            }

            all_progresses.insert(new_session_id.clone(), new_process.clone());

            (Some(new_session_id), new_process)
        };

        // Insert the LoginProcess into request extensions
        let extensions = request.extensions_mut();
        extensions.insert(process.clone());

        let future = self.inner.call(request);

        // Outgoing response
        Box::pin(async move {
            let mut response: Response = future.await?;

            if let Some(new_session_id) = set_cookie {
                response.headers_mut().insert(
                    "Set-Cookie",
                    HeaderValue::from_str(&format!(
                        "{}={}; Secure; HttpOnly; SameSite=Strict;",
                        COOKIE_KEY, new_session_id
                    ))
                    .expect("Invalid Set-Cookie value"),
                );
            }
            Ok(response)
        })
    }
}
