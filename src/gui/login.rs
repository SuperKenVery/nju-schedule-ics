use super::utils::Centered;
use super::utils::Result;
use crate::gui::utils::CustomError;
use crate::gui::utils::ResultExt;
use anyhow::Context;
use daisy_rsx::marketing::hero::Hero;
use derivative::Derivative;
use dioxus::{
    html::textarea::placeholder,
    prelude::{server_fn::error::NoCustomError, *},
};
use image::DynamicImage;

#[component]
pub fn Login() -> Element {
    let image_src: Signal<Option<String>> = use_signal(|| None);
    use_resource(move || async move {});
    rsx! {
        Centered {
            background: "https://authserver.nju.edu.cn/authserver/custom/images/back3.jpg",

            FieldSet {
                title: "登陆",

                InputField { name: "账号", input_type: "text", place_holder: "" }
                InputField { name: "密码", input_type: "password", place_holder: "" }
                InputField {
                    name: "验证码", input_type: "text", place_holder: "",

                    // if let Some(src) = image_src() {
                    //     rsx!{
                    //         img {
                    //             class: "badge badge-neutral badge-xs",
                    //             src: src
                    //         }
                    //     }
                    // }
                    if true {
                        p { "captcha image here" }
                    }
                }

            }
        }
    }
}

#[component]
fn FieldSet(title: String, children: Element) -> Element {
    rsx! {
        fieldset {
            class: "fieldset bg-base-200 border-base-300 rounded-box w-xs border p-4",

            legend { class: "fieldset-legend", title }
            {children}
        }
    }
}

#[component]
fn InputField(
    name: String,
    input_type: String,
    place_holder: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        label {
            class: "input",

            {name.clone()}
            input { type: input_type, class: "input", placeholder: place_holder.unwrap_or(name) }
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

    Ok(captcha.clone().into_bytes())
}
