use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use axum::extract::FromRequestParts;
use axum::{body::Body, extract::Request, response::Response};
use derivative::Derivative;
use dioxus::server::ServerFnError;
use futures_util::future::BoxFuture;
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use tokio::sync::Mutex;
use tower::{Layer, Service};
use tower_cookies::Cookie;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::adapters::traits::{Credentials, LoginSession, School};
use crate::server::state::ServerState;

/// A login process. Its lifecycle is:
/// - Lifecycle starts when the user access this website. The user gets a session ID now.
/// - Then the user selects a school API, and logins with username and password.
/// - After that, we associate the session ID with a [`Credentials`], storing that in DB.
/// - Now this session is done, and is removed from the HashMap in LoginProcessManager.
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub enum LoginProcess {
    Started {
        school_adapters: Arc<Mutex<HashMap<&'static str, Arc<dyn School>>>>,
    },
    SelectedSchool {
        school: Arc<dyn School>,
        session: Arc<dyn LoginSession>,
    },
    Finished {
        school: Arc<dyn School>,
        #[derivative(Debug = "ignore")]
        credentials: Arc<dyn Credentials>,
        cred_db_key: String,
    },
}

impl LoginProcess {
    /// Start a new session
    pub fn start(school_adapters: Arc<Mutex<HashMap<&'static str, Arc<dyn School>>>>) -> Self {
        Self::Started { school_adapters }
    }

    /// Set the school adapter for this session
    pub async fn select_school(&mut self, school_name: String) -> Result<()> {
        let Self::Started { school_adapters } = self else {
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
        *self = Self::SelectedSchool {
            school,
            session: login_session.into(),
        };

        Ok(())
    }

    /// Get the captcha image content
    pub async fn get_captcha(&self) -> Result<&DynamicImage> {
        let Self::SelectedSchool { session, .. } = self else {
            bail!("Not in SelectedSchool when calling `get_captcha`");
        };

        Ok(session.get_captcha())
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
        captcha_answer: String,
    ) -> Result<()> {
        let Self::SelectedSchool { school, session } = self else {
            bail!("Not in SelectedSchool when calling `login`");
        };

        let cred = session.login(username, password, captcha_answer).await?;
        let cred_db_key = session.save_cred_to_db(cred.clone()).await?;

        *self = Self::Finished {
            school: school.clone(),
            credentials: cred.into(),
            cred_db_key,
        };

        Ok(())
    }
}

impl<S> FromRequestParts<S> for LoginProcess {
    type Rejection = ServerFnError;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> impl Future<Output = std::result::Result<Self, Self::Rejection>> + Send {
        async move {
            Ok(parts
                .extensions
                .get::<Self>()
                .expect("No LoginProcess found in extension. Is `LoginProcessManagerLayer` setup?")
                .clone())
        }
    }
}

// === Layer ===
// Use when setting up routes

pub struct LoginProcessManagerLayer {}

impl<S> Layer<S> for LoginProcessManagerLayer {
    type Service = LoginProcessManager<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoginProcessManager {
            inner,
            all_processes: HashMap::new(),
        }
    }
}

// === Middleware ===

struct LoginProcessManager<S> {
    inner: S,
    all_processes: HashMap<String, LoginProcess>,
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
        const COOKIE_KEY: &'static str = "session_id";

        // Incoming request
        let cookies = request
            .extensions()
            .get::<Cookies>()
            .expect("Cannot get cookies. Is `CookieManagerLayer` configured?");
        let session_id = cookies.get(COOKIE_KEY);

        let (set_cookie, process) = if let Some(session_id) = session_id
            && let Some(process) = self.all_processes.get(session_id.to_string().as_str())
        {
            // Session found, insert into extensions
            (None, process.clone())
        } else {
            // Session invalid or not found, create a new session and insert into extensions
            let server_state = request
                .extensions()
                .get::<ServerState>()
                .expect("ServerState not found in extensions");
            let school_adapters = server_state.school_adapters.clone();
            let new_session_id = Uuid::new_v4().to_string();
            let new_process = LoginProcess::Started { school_adapters };

            self.all_processes
                .insert(new_session_id.clone(), new_process.clone());

            (Some(new_session_id), new_process)
        };

        // Insert the LoginProcess into request extensions
        let extensions = request.extensions_mut();
        extensions.insert(process.clone());

        let future = self.inner.call(request);

        // Outgoing response
        Box::pin(async move {
            let mut response: Response = future.await?;

            match set_cookie {
                Some(new_session_id) => {
                    let cookies = response
                        .extensions_mut()
                        .get_mut::<Cookies>()
                        .expect("ServerState not found in extensions");
                    cookies.add(Cookie::new(COOKIE_KEY, new_session_id));
                }
                None => {}
            }
            Ok(response)
        })
    }
}
