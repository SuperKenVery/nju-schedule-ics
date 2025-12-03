use crate::gui::utils::to_blob_url;

use super::super::app::Route;
use super::super::utils::{ButtonWithLoading, Hero};
use anyhow::Result;
use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let img_src = use_resource(move || async move {
        let image = get_captcha().await?;

        to_blob_url(&image)
    });

    let username = use_signal(|| "".to_string());
    let password = use_signal(|| "".to_string());
    let captcha_answer = use_signal(|| "".to_string());

    rsx! {
        Hero {
            image: "https://authserver.nju.edu.cn/authserver/custom/images/back3.jpg",

            FieldSet {
                onsubmit: move |event| {
                    debug!("FieldSet got event: {:#?}", event);
                },
                InputField { name: "账号", input_type: "text", place_holder: "", bind: username }
                InputField { name: "密码", input_type: "password", place_holder: "", bind: password }
                InputField {
                    name: "验证码", input_type: "text", place_holder: "", bind: captcha_answer,

                    match &*img_src.read_unchecked() {
                        Some(Ok(url)) => rsx! {
                            img {
                                class: "badge badge-neutral badge-xl p-0 px-0",
                                src: url.to_string()
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { {format!("加载失败：{:?}", e)} }
                        },
                        None => rsx!{
                            p { "验证码加载中" }
                        }
                    }
                }
                ButtonWithLoading {
                    class: "btn btn-neutral mt-4",
                    type: "submit",
                    onclick: move |_event| async move {
                        let _db_key = login_for_session(username(), password(), captcha_answer()).await?;

                        let nav = navigator();
                        nav.push(Route::ViewLink);

                        Ok(())
                    },
                    "登陆"
                }
            }
        }
    }
}

#[component]
fn FieldSet(onsubmit: Option<EventHandler<FormEvent>>, children: Element) -> Element {
    rsx! {
        form {
            onsubmit: move |event| {
                if let Some(handler) = onsubmit {
                    handler(event);
                }
            },
            fieldset {
                class: "fieldset bg-base-200 border-base-300 rounded-box w-lg border p-4",
                {children}
            }
        }
    }
}

#[component]
fn InputField(
    name: String,
    input_type: String,
    place_holder: Option<String>,
    bind: Signal<String>,
    children: Element,
) -> Element {
    rsx! {
        label {
            // class: "input-ghost input border border-current m-2 focus-within:bg-transparent",
            class: "input w-full",

            {name.clone()}
            input {
                type: input_type,
                class: "grow",
                placeholder: place_holder.unwrap_or(name),
                oninput: move |event| {
                    bind.set(event.data().value());
                }
            }
            {children}
        }
    }
}

#[cfg(feature = "server")]
use crate::adapters::login_process::LoginProcess;

#[get("/api/get_captcha", session: LoginProcess)]
async fn get_captcha() -> Result<Vec<u8>> {
    use std::io::Cursor;

    let captcha = session.get_captcha().await?;

    // Convert to PNG
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    captcha.write_to(&mut cursor, image::ImageFormat::Png)?;

    Ok(png_bytes)
}

#[post("/api/login", session: LoginProcess)]
async fn login_for_session(
    username: String,
    password: String,
    captcha_answer: String,
) -> Result<String> {
    let cred_db_key = session.login(username, password, captcha_answer).await?;

    Ok(cred_db_key)

    // TODO: Cleanup the leftover in LoginProcessManager.all_progresses
}
