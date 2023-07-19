use aes::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit},
    Aes128,
};
use base64::{engine::general_purpose, Engine as _};
use cbc;
use core::ops::Deref;
use std::{collections::HashMap, error::Error, future::Future, hash::Hash};
use tl::{self, HTMLTag, VDom, VDomGuard};

fn encrypt(password: &str, salt: &str) -> String {
    type Aes128CbcEnc = cbc::Encryptor<Aes128>;
    let cipher = Aes128CbcEnc::new(salt.as_bytes().into(), &[1u8; 16].into());

    let ct = cipher.encrypt_padded_vec_mut::<Pkcs7>(password.as_bytes());
    let b64 = general_purpose::STANDARD.encode(ct);

    b64
}

fn extract_context(document: &VDom) -> Option<HashMap<String, String>> {
    let mut context = HashMap::new();

    let parser = document.parser();

    let form = document.get_element_by_id("casLoginForm")?.get(parser)?;
    let children = form.children()?.all(parser);

    for child in children {
        let Some(tag)=child.as_tag() else{
            continue;
        };

        if tag.name() != "input" {
            continue;
        }

        let attr = tag.attributes();

        let Some(Some(tag_type))=attr.get("type") else {
            continue;
        };

        if tag_type.as_utf8_str() != "hidden" {
            continue;
        }

        if let Some(Some(name)) = attr.get("name") {
            let value = attr.get("value")??;
            context.insert(
                name.as_utf8_str().into_owned(),
                value.as_utf8_str().into_owned(),
            );
        } else if let Some(Some(id)) = attr.get("id") {
            let value = attr.get("value")??;
            context.insert(
                id.as_utf8_str().into_owned(),
                value.as_utf8_str().into_owned(),
            );
        }
    }

    Some(context)
}

#[derive(Debug)]
struct LoginCredential {
    castgc: String,
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

        let get_cookie_response = client
            .get("https://authserver.nju.edu.cn/authserver/login")
            .send()
            .await?;

        let login_page_response = client
            .get("https://authserver.nju.edu.cn/authserver/login")
            .send()
            .await?;

        let login_page_raw = login_page_response.text().await?;
        let login_page = unsafe { tl::parse_owned(login_page_raw, tl::ParserOptions::default())? };

        let context = extract_context(login_page.get_ref()).unwrap();

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

        let captcha_answer = captcha(captcha_content_buf.to_vec()).await;

        let encrypted_password = encrypt(password, &context["pwdDefaultEncryptSalt"]);

        let mut form = context.clone();
        form.insert("username".to_string(), username.to_string());
        form.insert("password".into(), encrypted_password);
        form.insert("captchaResponse".into(), captcha_answer);

        let login_response = client
            .post("https://authserver.nju.edu.cn/authserver/login")
            .form(&form)
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
        println!("test login begin");
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
                    use_kitty: false,
                    use_iterm: false,
                    ..Default::default()
                },
            )
            .unwrap();

            let mut captcha = String::new();
            io::stdin().read_line(&mut captcha).unwrap();

            captcha
        })
        .await;

        println!("Login done running");
        let Ok(l)=l else{
            let l=l.unwrap_err();
            panic!("{}",format!("Result is err: {:#?}",l));
        };

        assert_ne!(l.castgc, "");
    }
}
