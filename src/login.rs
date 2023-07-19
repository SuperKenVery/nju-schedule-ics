use aes::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit},
    Aes128,
};
use base64::{engine::general_purpose, Engine as _};
use cbc;
use core::ops::Deref;
use skyscraper::{html, xpath};
use std::{collections::HashMap, error::Error, future::Future, hash::Hash};

fn encrypt(password: &str, salt: &str) -> String {
    type Aes128CbcEnc = cbc::Encryptor<Aes128>;
    let cipher = Aes128CbcEnc::new(salt.as_bytes().into(), &[1u8; 16].into());

    let ct = cipher.encrypt_padded_vec_mut::<Pkcs7>(password.as_bytes());
    let b64 = general_purpose::STANDARD.encode(ct);

    b64
}

#[derive(Debug)]
struct LoginCredential {
    castgc: String,
}

trait GetInfoExt {
    fn get_value(&self, selector: &str) -> Result<String, Box<dyn Error>>;
}

impl GetInfoExt for html::HtmlDocument {
    fn get_value(&self, selector: &str) -> Result<String, Box<dyn Error>> {
        let form_selector = xpath::parse("//*[@id=\"casLoginForm\"]").unwrap();
        let form = form_selector.apply(self)?[0];
        println!("Form is {:#?}", form);

        let xpath_expr = xpath::parse(selector)?;
        let nodes = xpath_expr.apply_to_node(self, form)?;

        if nodes.len() != 1 {
            println!(
                "{}",
                format!(
                    "When matching {}, found {}!=1 elements, {:#?}",
                    selector,
                    nodes.len(),
                    nodes
                )
            );
            for node in &nodes {
                let a = HashMap::new();
                let attrs = node.get_attributes(self).unwrap_or(&a);
                let text = node.get_all_text(self).unwrap_or("".to_string());

                println!("Node text={:#?} \n attrs={:#?}", text, attrs);
            }
            return Err(format!("Found {}!=1 elements, {:#?}", nodes.len(), nodes).into());
        }

        let node = nodes[0];
        let Some(attrs)=node.get_attributes(self) else{
            return Err("Cannot get attributes".into());
        };

        let Some(val)=attrs.get("value") else{
            return Err("No `value` attr".into());
        };

        Ok(val.to_owned())
    }
}

impl LoginCredential {
    pub async fn new<F>(
        username: &str,
        password: &str,
        captcha: impl Fn(Vec<u8>) -> F,
    ) -> Result<LoginCredential, Box<dyn Error>>
    where
        F: Future<Output = String>,
    {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6.1 Safari/605.1.15".parse().unwrap());
        headers.insert("origin", "https://authserver.nju.edu.cn".parse().unwrap());
        headers.insert(
            "referer",
            "https://authserver.nju.edu.cn/authserver/login"
                .parse()
                .unwrap(),
        );

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap();

        println!("get cookie...");
        let get_cookie_response = client
            .get("https://authserver.nju.edu.cn/authserver/login")
            .send()
            .await?;

        println!("login page...");
        let login_page_response = client
            .get("https://authserver.nju.edu.cn/authserver/login")
            .send()
            .await?;

        let login_page_raw = login_page_response.text().await?;
        println!("Page is {}", login_page_raw);
        let login_page = html::parse(&login_page_raw)?;

        let lt = login_page
            .get_value("//*[@id=\"casLoginForm\"]/input[@name=\"lt\"]")
            .unwrap();
        let dllt = "mobileLogin".to_string();
        let execution = login_page
            .get_value("//*[@id=\"casLoginForm\"]/input[@name=\"execution\"]")
            .unwrap();
        let eventid = login_page
            .get_value("//*[@id=\"casLoginForm\"]/input[@name=\"_eventId\"]")
            .unwrap();
        let rmshown = login_page
            .get_value("//*[@id=\"casLoginForm\"]/input[@name=\"rmShown\"]")
            .unwrap();
        let salt = login_page
            .get_value("//*[@id=\"pwdDefaultEncryptSalt\"]")
            .unwrap();

        let need_captcha_response=client
            .get(format!("https://authserver.nju.edu.cn/authserver/needCaptcha.html?username={}&pwdEncrypt2=pwdEncryptSalt",username))
            .send().await?;
        let captcha_content = client
            .get("https://authserver.nju.edu.cn/authserver/captcha.html")
            .send()
            .await?
            .bytes()
            .await?;
        let captcha_content_buf = captcha_content.deref();

        println!("Asking for captcha...");
        let captcha_answer = captcha(captcha_content_buf.to_vec()).await;

        let encrypted_password = encrypt(password, &salt);

        let login_response = client
            .post("https://authserver.nju.edu.cn/authserver/login")
            .form(&HashMap::from([
                ("username", username),
                ("password", &encrypted_password),
                ("captchaResponse", &captcha_answer),
                ("lt", &lt),
                ("dllt", &dllt),
                ("execution", &execution),
                ("_eventId", &eventid),
                ("rmShown", &rmshown),
            ]))
            .send()
            .await?;

        for cookie in login_response.cookies() {
            if cookie.name() == "CASTGC" {
                println!("Castgc is {}", cookie.value());
                return Ok(LoginCredential {
                    castgc: cookie.value().to_string(),
                });
            }
        }

        Err("CASTGC Not found".into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use image::{io::Reader, *};
    use std::{io, io::Cursor};
    use tokio;
    use viuer::{print, Config};

    #[tokio::test]
    async fn login_works() {
        let l = LoginCredential::new("test_account", "test_password", |buf| async {
            let image = Reader::new(Cursor::new(buf)).with_guessed_format().unwrap();
            let img = image.decode().unwrap();

            print(
                &img,
                &Config {
                    width: None,
                    height: None,
                    x: 0,
                    y: 0,
                    ..Default::default()
                },
            )
            .unwrap();

            let mut captcha = String::new();
            io::stdin().read_line(&mut captcha).unwrap();

            captcha
        })
        .await;

        let Ok(l)=l else{
            let l=l.unwrap_err();
            panic!("{}",format!("Result is err: {:#?}",l));
        };

        assert_ne!(l.castgc, "");
    }
}
