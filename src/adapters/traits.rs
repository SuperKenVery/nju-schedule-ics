use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use diesel::{Connection, Insertable, RunQueryDsl, Selectable, SqliteConnection, Table};
use downcast_rs::{impl_downcast, Downcast};
use image::DynamicImage;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use std::any::Any;

/// An adapter for a school API.
///
/// A physical school can have multiple APIs, which corresponds to
/// multiple [`School`]s here
pub trait School: Login + Courses {
    fn new(db: SqliteConnection) -> Self
    where
        Self: Sized;
}

/// Supports logging in to the school.
#[async_trait]
pub trait Login {
    /// Create a new login session.
    async fn new_login_session(&self) -> Result<Box<dyn LoginSession>>;
    /// Query login credential in database
    async fn get_cred_from_db(&self) -> Option<Box<dyn Credentials>>;
    /// Create an HTTP client given the login credentials.
    async fn create_authenticated_client(
        &self,
        credentials: Box<dyn Credentials>,
    ) -> Result<ClientWithMiddleware>;
}

/// A course
pub struct Course {
    /// Course name
    pub name: String,
    /// All times of course, including each one across the semester.
    /// Format is `Vec<(start_time, end_time)>`.
    pub time: Vec<(DateTime<Utc>, DateTime<Utc>)>,
    /// The location of this course.
    pub location: Option<String>,
    /// The campus of this course
    pub campus: Option<String>,
    /// Additional notes.
    ///
    /// This would be in the notes area of calendar event, and you can
    /// include anything like notes, teacher, notice or whatsoever.
    ///
    /// When displayed, the vec of string will be concatenated with
    /// two new lines (thus an empty line between each string)
    pub notes: Vec<String>,
}

/// Supports getting courses from school.
pub trait Courses {
    /// Get courses
    fn courses(&self, client: &Client) -> Vec<Course>;
}

/// The login credential for a school.
///
/// It should implement [`Insertable`] so that it could be
/// saved to a database.
pub trait Credentials: Downcast {
    /// Save the credential to database
    fn save_to_db(&self) -> Result<()>;
}
impl_downcast!(Credentials);

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
pub trait LoginSession {
    /// Get the content of captcha image
    fn get_captcha(&self) -> &DynamicImage;

    /// Send the login request
    async fn login(
        &self,
        username: String,
        password: String,
        captcha_answer: String,
    ) -> Result<Box<dyn Credentials>>;
}

impl<S, T> Credentials for S
where
    T: Table,
    S: Savable<T = T> + 'static,
{
    fn save_to_db(&self) -> Result<()> {
        diesel::insert_into(self.table())
            .values(self)
            .execute(self.connection())?;

        Ok(())
    }
}

pub trait Savable {
    /// The table type
    type T: Table;
    type C: Connection;

    fn table(&self) -> Self::T;
    fn connection(&self) -> &mut Self::C;
}
