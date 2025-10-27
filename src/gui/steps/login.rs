use std::io::Cursor;

use crate::gui::utils::to_blob_url;

use super::super::app::Route;
use super::super::utils::{
    ButtonWithLoading, Centered, ClientState, CustomError, Hero, Result, ResultExt,
};
use anyhow::{Context, anyhow};
use daisy_rsx::Button;
use derivative::Derivative;
use dioxus::{
    html::textarea::placeholder,
    prelude::{server_fn::error::NoCustomError, *},
};
use image::DynamicImage;
use tracing::info;

#[component]
pub fn Login() -> Element {
    use js_sys::{Array, Uint8Array};
    use web_sys::{Blob, Url};

    let mut client_state = use_context::<Signal<ClientState>>();
    let img_src = use_resource(move || async move {
        let image = get_captcha(
            client_state
                .read()
                .session_id
                .clone()
                .context("No session found")?,
        )
        .await?;

        Ok::<_, anyhow::Error>(to_blob_url(&image)?)
    });

    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut captcha_answer = use_signal(|| "".to_string());

    rsx! {
        Hero {
            image: "https://authserver.nju.edu.cn/authserver/custom/images/back3.jpg",

            FieldSet {
                onsubmit: move |event| {
                    info!("FieldSet got event: {:#?}", event);
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
                    onclick: move |event| async move {
                        if let Some(session_id) = client_state().session_id {
                            let db_key = login_for_session(session_id, username(), password(), captcha_answer()).await?;
                            (*(client_state.write())).db_key = Some(db_key);

                            let nav = navigator();
                            nav.push(Route::ViewLink);
                        }

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

#[server]
async fn get_captcha(session_id: String) -> Result<Vec<u8>, ServerFnError<CustomError>> {
    use crate::server::state::{ServerState, UnfinishedLoginSession};
    use uuid::Uuid;

    let FromContext(state): FromContext<ServerState> = extract().await.to_sfn()?;
    let sessions = state.unfinished_login_sessions.lock().await;
    let session = sessions
        .get(&session_id)
        .context("No such session")
        .to_sfn()?;

    let captcha = session.get_captcha().await.to_sfn()?;

    // Convert to PNG
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    captcha
        .write_to(&mut cursor, image::ImageFormat::Png)
        .to_sfn()?;

    Ok(png_bytes)
}

#[server]
async fn login_for_session(
    session_id: String,
    username: String,
    password: String,
    captcha_answer: String,
) -> Result<String, ServerFnError<CustomError>> {
    use crate::server::state::{ServerState, UnfinishedLoginSession};
    let FromContext(state): FromContext<ServerState> = extract().await.to_sfn()?;
    let mut sessions = state.unfinished_login_sessions.lock().await;
    let session = sessions
        .get_mut(&session_id)
        .context("No such session")
        .to_sfn()?;

    session
        .login(username, password, captcha_answer, state.db.clone())
        .await
        .to_sfn()?;

    let UnfinishedLoginSession::Finished { cred_db_key, .. } = session else {
        return Err(anyhow!("Login didn't finish")).to_sfn();
    };

    Ok(cred_db_key.clone())
}
