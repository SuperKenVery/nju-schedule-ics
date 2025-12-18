use super::NJUUndergradAdaptor;

use crate::adapters::traits::{Credentials, Login, LoginSession};
use aes::{
    Aes128,
    cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7},
};
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use derivative::Derivative;

use image::{DynamicImage, ImageReader};
use reqwest::{Client, Url, cookie::Jar};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use skyscraper::html::{self};
use skyscraper::xpath::{self, XpathItemTree, grammar::data_model::XpathItem};
use sqlx::SqlitePool;
use sqlx::prelude::FromRow;
use std::sync::Arc;
use std::{collections::HashMap, io::Cursor};
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;
// use xee_xpath::{DocumentHandle, Documents, Queries, Query};

#[async_trait]
impl Login for NJUUndergradAdaptor {
    async fn new_login_session(&self) -> Result<Box<dyn LoginSession>> {
        Ok(Box::new(Session::new(self.connection.clone()).await?))
    }

    async fn get_cred_from_db(&self, session_id: &str) -> Option<Box<dyn Credentials>> {
        let connection = self.connection.lock().await;

        let mut cred = sqlx::query_as::<_, LoginCredential>("SELECT * FROM castgc WHERE key = ?")
            .bind(session_id)
            .fetch_one(&*connection)
            .await
            .ok()?;
        cred.last_access = chrono::Local::now().naive_local();

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
    db: Arc<Mutex<SqlitePool>>,
    id: String,
    client: reqwest::Client,
    #[derivative(Debug = "ignore")]
    captcha: DynamicImage,
    context: HashMap<String, String>,
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
                let doc = login_response
                    .text()
                    .await
                    .context("Getting login failed response")?
                    .xptree()
                    .context("Parsing login fail response")?;

                let reason = doc
                    .xpath("//form[@id='casLoginForm']//span[@class='auth_error']/text()")?
                    .first()
                    .context("Cannot get login fail reason")?
                    .to_string();

                Err(anyhow!(reason))
            }
        }
    }

    fn session_id(&self) -> &str {
        &self.id
    }

    async fn save_cred_to_db(&self, cred: Box<dyn Credentials>) -> Result<String> {
        let cred: Box<LoginCredential> = cred
            .downcast()
            .map_err(|_| anyhow!("Got invalid credential when saving to db, downcasting failed"))?;
        let db_key = cred.key.clone();

        let connection = self.db.lock().await;
        let _inserted =
            sqlx::query("INSERT INTO castgc (key, value, last_access) VALUES ($1, $2, $3)")
                .bind(&db_key)
                .bind(cred.value)
                .bind(cred.last_access)
                .execute(&*connection)
                .await?;

        Ok(db_key)
    }
}

impl Session {
    /// Create a login session
    ///
    /// by requesting the login page
    pub async fn new(db: Arc<Mutex<SqlitePool>>) -> Result<Self> {
        let (client, _jar) = build_client().await?;
        let login_page_response = client
            .get("https://authserver.nju.edu.cn/authserver/login")
            .send()
            .await?;

        let context = extract_context(login_page_response.text().await?.xptree()?)?;

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
pub fn extract_context(login_page: XpathItemTree) -> Result<HashMap<String, String>> {
    let variables = login_page.xpath("//form[@id='casLoginForm']/input")?;

    let mut context = HashMap::new();

    for variable in variables.into_iter() {
        let node = &variable.as_node()?.as_tree_node()?.data.as_element_node()?;
        let (Some("hidden"), Some(name), Some(value)) = (
            node.get_attribute("type"),
            node.get_attribute("name").or(node.get_attribute("id")),
            node.get_attribute("value"),
        ) else {
            continue;
        };
        info!("Context: adding name={}, value={}", name, value);
        context.insert(name.to_string(), value.to_string());
    }

    Ok(context)
}

/// Encrypt the password
pub fn encrypt(password: &str, salt: &str) -> String {
    type Aes128CbcEnc = cbc::Encryptor<Aes128>;
    let iv = "a".repeat(16).into_bytes();
    let cipher = Aes128CbcEnc::new(salt.as_bytes().into(), iv.as_slice().into());

    let ct =
        cipher.encrypt_padded_vec_mut::<Pkcs7>(("a".repeat(64) + password).into_bytes().as_slice());

    general_purpose::STANDARD.encode(ct)
}

/// Build the network client with appropriate headers needed for login page
///
/// This client isn't logged in; it is used for logging in.
pub async fn build_client() -> Result<(Client, Arc<Jar>)> {
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

#[derive(FromRow, Clone)]
pub struct LoginCredential {
    /// The session ID
    pub key: String,
    /// The CASTGC cookie
    pub value: String,
    /// Time last accessed
    pub last_access: chrono::NaiveDateTime,
}

// === Utils for using xpath easier ===

pub trait ToXpathTree {
    fn xptree(&self) -> Result<XpathItemTree>;
}

impl ToXpathTree for String {
    fn xptree(&self) -> Result<XpathItemTree> {
        let doc = html::parse(self.as_str())?;
        let xpath_item_tree = XpathItemTree::from(&doc);
        Ok(xpath_item_tree)
    }
}

pub trait XpathExt {
    fn xpath(&self, query: &'static str) -> Result<Vec<XpathItem<'_>>>;
}

impl XpathExt for XpathItemTree {
    fn xpath(&self, query: &'static str) -> Result<Vec<XpathItem<'_>>> {
        let xpath_query = xpath::parse(query)?;
        let item_set = xpath_query.apply(self)?;
        Ok(item_set.into_iter().collect())
    }
}
