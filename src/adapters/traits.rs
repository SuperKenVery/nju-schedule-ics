use super::course::Course;
use anyhow::Result;
use async_trait::async_trait;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;
use image::DynamicImage;
use reqwest_middleware::ClientWithMiddleware;
use sqlx::SqlitePool;
use std::{fmt::Debug, sync::Arc};
use tokio::sync::Mutex;

/// An adapter for a school API.
///
/// A physical school can have multiple APIs, which corresponds to
/// multiple [`School`]s here
#[async_trait]
pub trait School: Login + CoursesProvider + CalendarHelper + Send + Sync + Debug {
    /// Create an instance. Do database migrations if needed.
    async fn new(db: Arc<Mutex<SqlitePool>>) -> Self
    where
        Self: Sized;

    /// The name for this api adapter.
    fn adapter_name(&self) -> &str;
}

/// Supports logging in to the school.
#[async_trait]
pub trait Login {
    /// Create a new login session.
    async fn new_login_session(&self) -> Result<Box<dyn LoginSession>>;
    /// Query login credential in database
    async fn get_cred_from_db(&self, db_key: &str) -> Option<Box<dyn Credentials>>;
    /// Create an HTTP client given the login credentials.
    ///
    /// This client is logged in.
    async fn create_authenticated_client(
        &self,
        credentials: Box<dyn Credentials>,
    ) -> Result<ClientWithMiddleware>;
}

/// Supports getting courses from school.
#[async_trait]
pub trait CoursesProvider {
    /// Get courses
    async fn courses(&self, client: &ClientWithMiddleware) -> Result<Vec<Course>>;
}

/// The login credential for a school.
pub trait Credentials: Downcast + Send + Sync + DynClone {}
impl_downcast!(Credentials);
dyn_clone::clone_trait_object!(Credentials);

impl<T> Credentials for T where T: Downcast + Send + Sync + DynClone {}

/// A login session for the user to login.
///
/// The typical workflow is:
/// - **Start a login session.**
///   You request the school's login page, get a cookie and captcha image.
/// - **User submits username, password and captcha result.**
///   To display the captcha to user, we need the first step.
/// - **Finish login.**
///   You request the school's login page to finish the login.
#[async_trait]
pub trait LoginSession: Send + Sync + Debug {
    /// Get the content of captcha image
    fn get_captcha(&self) -> &DynamicImage;

    /// Send the login request
    async fn login(
        &self,
        username: String,
        password: String,
        captcha_answer: String,
    ) -> Result<Box<dyn Credentials>>;

    /// Get the session ID.
    ///
    /// This is set as a cookie to distinguish different logins.
    fn session_id(&self) -> &str;

    /// Save credential to DB.
    ///
    /// Returns the key in DB. When we fetch courses, we use this key to
    /// get the credentials.
    async fn save_cred_to_db(&self, cred: Box<dyn Credentials>) -> Result<String>;
}

/// Helps generating iCalendar calendar and events
pub trait CalendarHelper {
    /// The name of the school.
    fn school_name(&self) -> &str;
}
