use chrono::{DateTime, TimeZone};
use diesel::{Insertable, Selectable, Table};
use image::DynamicImage;
use reqwest::Client;

/// An adapter for a school API.
///
/// A physical school can have multiple APIs, which corresponds to
/// multiple [`School`]s here
pub trait School: Login + Courses {}

/// Supports logging in to the school.
pub trait Login {
    /// Create a new login session.
    fn new_login_session(&self) -> Box<dyn LoginSession>;
    /// Query login credential in database
    fn get_cred_from_db(&self) -> Option<Box<dyn Credentials>>;
    /// Create an HTTP client given the login credentials.
    fn authenticated_client(&self, credentials: Box<dyn Credentials>) -> Result<Client>;
}

/// A course
pub struct Course<T: TimeZone> {
    /// Course name
    pub name: String,
    /// All times of course, including each one across the semester.
    /// Format is `Vec<(start_time, end_time)>`.
    pub time: Vec<(DateTime<T>, DateTime<T>)>,
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
pub trait Credentials {
    /// Save the credential to database
    fn save_to_db(&self) -> Result<()>;
}

/// A login session for the user to login.
///
/// The typical workflow is:
/// - **Start a login session.**
///   You request the school's login page, get a cookie and captcha image.
/// - **User submits username, password and captcha result.**
///   To display the captcha to user, we need the first step.
/// - **Finish login.**
///   You request the school's login page to finish the login.
pub trait LoginSession {
    /// Get the content of captcha image
    fn get_captcha(&self) -> DynamicImage;

    /// Send the login request
    fn login(&self, username: String, password: String) -> Box<dyn Credentials>;

    /// Get the table to store credentials
    fn db_table(&self) -> Box<dyn Table>;
}

#[test]
mod test {
    use diesel::prelude::*;
    fn aaa() {
        diesel::insert_into(target).values(records)
    }
}
