use super::NJUBatchelorAdaptor;
use crate::adapters::traits::{Credentials, Login, LoginSession, Savable};
use aes::{
    cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit},
    Aes128,
};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, ImageReader};
use reqwest::{cookie::Jar, Client, Url};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::sync::Arc;
use std::{collections::HashMap, io::Cursor};
use xee_xpath::{DocumentHandle, Documents, Queries, Query};

#[async_trait]
impl Login for NJUBatchelorAdaptor {
    async fn new_login_session(&self) -> Result<Box<dyn LoginSession>> {
        Ok(Box::new(Session::new().await?))
    }

    async fn get_cred_from_db(&self) -> Option<Box<dyn Credentials>> {
        todo!()
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

        let credentials: Box<LoginCredentials> = credentials
            .downcast()
            .map_err(|_| anyhow!("Invalid login credentials (failed to downcast)"))?;
        jar.add_cookie_str(
            format!("CASTGC={}", credentials.castgc).as_str(),
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

#[derive(Debug)]
pub struct Session {
    client: reqwest::Client,
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

        match login_response.cookies().find(|x| x.name() == "CASTGC") {
            Some(castgc) => Ok(Box::new(LoginCredentials {
                castgc: castgc.value().to_string(),
            })),
            None => {
                let (mut err_page, doc_handle) = {
                    let mut doc = Documents::new();
                    let doc_handle = doc.add_string(
                        "https://authserver.nju.edu.cn"
                            .try_into()
                            .context("Parsing login failed response")?,
                        &login_response
                            .text()
                            .await
                            .context("Parsing login failed response")?,
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
}

impl Session {
    /// Create a login session
    ///
    /// by requesting the login page
    async fn new() -> Result<Self> {
        let (client, _jar) = build_client().await?;
        let login_page_response = client
            .get("https://authserver.nju.edu.cn/authserver/login")
            .send()
            .await?;

        let login_page_raw = login_page_response.text().await?;
        let (mut login_page, doc_handle) = {
            let mut doc = Documents::new();
            let doc_handle =
                doc.add_string("https://authserver.nju.edu.cn".try_into()?, &login_page_raw)?;
            (doc, doc_handle)
        };

        let context = extract_context(&mut login_page, doc_handle)?;

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
            client,
            captcha: captcha_image,
            context,
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

pub struct LoginCredentials {
    castgc: String,
}

// impl Credentials for LoginCredentials {
//     fn save_to_db(&self) -> Result<()> {
//         todo!()
//     }
// }

impl Savable for LoginCredentials {
    type T;

    type C;

    fn table(&self) -> Self::T {
        todo!()
    }

    fn connection(&self) -> &mut Self::C {
        todo!()
    }
}
