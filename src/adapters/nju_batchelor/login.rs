use super::db_schema::castgc::dsl::{castgc, key};
use super::NJUBatchelorAdaptor;
use crate::adapters::traits::{Credentials, Login, LoginSession};
use aes::{
    cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit},
    Aes128,
};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use derivative::Derivative;
use diesel::SelectableHelper;
use diesel::{
    prelude::QueryableByName, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl,
    Selectable, SqliteConnection,
};
use image::{DynamicImage, ImageReader};
use reqwest::{cookie::Jar, Client, Url};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io::Cursor};
use uuid::Uuid;
use xee_xpath::{DocumentHandle, Documents, Queries, Query};

#[async_trait]
impl Login for NJUBatchelorAdaptor {
    async fn new_login_session(&self) -> Result<Box<dyn LoginSession>> {
        Ok(Box::new(Session::new(self.connection.clone()).await?))
    }

    async fn get_cred_from_db(&self, session_id: &str) -> Option<Box<dyn Credentials>> {
        let mut connection = self.connection.lock().unwrap();

        let queried = castgc
            .filter(key.eq(session_id))
            .first::<RowLoginCredential>(&mut *connection)
            .ok()?;
        let cred: LoginCredential = queried.into();

        Some(Box::new(cred))
    }

    async fn create_authenticated_client(
        &self,
        credentials: Box<dyn Credentials>,
    ) -> Result<ClientWithMiddleware> {
        let jar = Arc::new(Jar::default());

        let client = reqwest_middleware::ClientBuilder::new(
            reqwest::ClientBuilder::new()
                .cookie_provider(jar.clone())
                .user_agent("nju-schedule-ics")
                .timeout(std::time::Duration::from_secs(10))
                .build()?,
        )
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .build();

        let credentials: Box<LoginCredential> = credentials
            .downcast()
            .map_err(|_| anyhow!("Invalid login credentials (failed to downcast)"))?;
        jar.add_cookie_str(
            format!("CASTGC={}", credentials.value).as_str(),
            &Url::parse("https://authserver.nju.edu.cn").unwrap(),
        );

        let _ = client
            .get("https://ehall.nju.edu.cn/appShow?appId=4770397878132218")
            .send()
            .await?
            .text()
            .await?;

        Ok(client)
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Session {
    id: String,
    client: reqwest::Client,
    captcha: DynamicImage,
    context: HashMap<String, String>,
    #[derivative(Debug = "ignore")]
    db: Arc<Mutex<SqliteConnection>>,
}

#[async_trait]
impl LoginSession for Session {
    fn get_captcha(&self) -> &DynamicImage {
        &self.captcha
    }

    async fn login(
        &self,
        username: String,
        password: String,
        captcha_answer: String,
    ) -> Result<Box<dyn crate::adapters::traits::Credentials>> {
        let encrypted_password = encrypt(
            &password,
            self.context
                .get("pwdDefaultEncryptSalt")
                .ok_or(anyhow!("Failed to find password encryption salt"))?,
        );

        let mut form = self.context.clone();
        form.insert("username".to_string(), username);
        form.insert("password".to_string(), encrypted_password);
        form.insert("captchaResponse".to_string(), captcha_answer);
        form.insert("dllt".to_string(), "mobileLogin".to_string());

        let login_response = self
            .client
            .post("https://authserver.nju.edu.cn/authserver/login")
            .form(&form)
            .send()
            .await?;

        let found_castgc = login_response.cookies().find(|x| x.name() == "CASTGC");
        match found_castgc {
            Some(castgc_cookie) => Ok(Box::new(LoginCredential {
                key: self.id.clone(),
                value: castgc_cookie.value().to_string(),
                last_access: chrono::Local::now().naive_local(),
            })),
            // Login failed, try to get reason from webpage
            None => {
                let response_text = &login_response
                    .text()
                    .await
                    .context("Parsing login failed response")?;
                let (mut err_page, doc_handle) = {
                    let mut doc = Documents::new();
                    let doc_handle = doc.add_string(
                        "https://authserver.nju.edu.cn"
                            .try_into()
                            .context("Parsing login failed response")?,
                        &response_text,
                    )?;

                    (doc, doc_handle)
                };
                let queries = Queries::default();
                let reason = queries
                    .one(
                        "//form[@id='casLoginForm']/span[@class='auth_error']/text()",
                        |_, item| Ok(item.try_into_value::<String>()?),
                    )
                    .context("Building xpath query to get login fail reason")?;
                let reason = reason
                    .execute(&mut err_page, doc_handle)
                    .context("Getting login fail reason")?;

                Err(anyhow!(reason))
            }
        }
    }

    fn session_id(&self) -> &str {
        &self.id
    }

    fn save_cred_to_db(&self, cred: Box<dyn Credentials>) -> Result<()> {
        let cred: Box<LoginCredential> = cred
            .downcast()
            .map_err(|_| anyhow!("Got invalid credential when saving to db, downcasting failed"))?;
        let mut connection = self.db.lock().unwrap();
        let connection_ref: &mut SqliteConnection = &mut *connection;

        let _inserted = diesel::insert_into(super::db_schema::castgc::dsl::castgc)
            .values(*cred)
            .execute(connection_ref)?;

        Ok(())
    }
}

impl Session {
    /// Create a login session
    ///
    /// by requesting the login page
    async fn new(db: Arc<Mutex<SqliteConnection>>) -> Result<Self> {
        let (client, _jar) = build_client().await?;
        let login_page_response = client
            .get("https://authserver.nju.edu.cn/authserver/login")
            .send()
            .await?;

        let context = {
            let login_page_raw = login_page_response.text().await?;
            let (mut login_page, doc_handle) = {
                let mut doc = Documents::new();
                let doc_handle =
                    doc.add_string("https://authserver.nju.edu.cn".try_into()?, &login_page_raw)?;
                (doc, doc_handle)
            };
            extract_context(&mut login_page, doc_handle)?
        };

        let captcha_content = client
            .get("https://authserver.nju.edu.cn/authserver/captcha.html")
            .send()
            .await?
            .bytes()
            .await?;
        let captcha_image = ImageReader::new(Cursor::new(captcha_content))
            .with_guessed_format()?
            .decode()?;

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            client,
            captcha: captcha_image,
            context,
            db,
        })
    }
}

/// Extract some attributes on the page needed for POST requests.
fn extract_context(
    login_page: &mut Documents,
    handle: DocumentHandle,
) -> Result<HashMap<String, String>> {
    let queries = Queries::default();

    let q_names = queries.many("//form[@id='casLoginForm']/input/@name", |_, item| {
        Ok(item.try_into_value::<String>()?)
    })?;
    let q_values = queries.many("//form[@id='casLoginForm']/input/@value", |_, item| {
        Ok(item.try_into_value::<String>()?)
    })?;

    let names = q_names.execute(login_page, handle)?;
    let values = q_values.execute(login_page, handle)?;

    let mut context = HashMap::new();

    for (name, value) in names.into_iter().zip(values.into_iter()) {
        context.insert(name, value);
    }

    Ok(context)
}

/// Encrypt the password
fn encrypt(password: &str, salt: &str) -> String {
    type Aes128CbcEnc = cbc::Encryptor<Aes128>;
    let iv = "a".repeat(16).into_bytes();
    let cipher = Aes128CbcEnc::new(salt.as_bytes().into(), iv.as_slice().into());

    let ct =
        cipher.encrypt_padded_vec_mut::<Pkcs7>(("a".repeat(64) + password).into_bytes().as_slice());
    let b64 = general_purpose::STANDARD.encode(ct);

    b64
}

/// Build the network client with appropriate headers needed for login page
///
/// This client isn't logged in; it is used for logging in.
async fn build_client() -> Result<(Client, Arc<Jar>)> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6.1 Safari/605.1.15".try_into().unwrap());
    headers.insert(
        "origin",
        "https://authserver.nju.edu.cn".try_into().unwrap(),
    );
    headers.insert(
        "referer",
        "https://authserver.nju.edu.cn/authserver/login"
            .try_into()
            .unwrap(),
    );

    let jar = Arc::new(Jar::default());
    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .cookie_provider(jar.clone())
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    Ok((client, jar))
}

#[derive(Insertable)]
#[diesel(table_name = super::db_schema::castgc)]
pub struct LoginCredential {
    /// The session ID
    key: String,
    /// The CASTGC cookie
    value: String,
    /// Time last accessed
    last_access: chrono::NaiveDateTime,
}

#[derive(Queryable)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct RowLoginCredential {
    id: Option<i32>,
    /// The session ID
    key: String,
    /// The CASTGC cookie
    value: String,
    /// Time last accessed
    last_access: chrono::NaiveDateTime,
}

impl Credentials for LoginCredential {}

impl From<RowLoginCredential> for LoginCredential {
    fn from(cred: RowLoginCredential) -> Self {
        Self {
            key: cred.key,
            value: cred.value,
            last_access: cred.last_access,
        }
    }
}
