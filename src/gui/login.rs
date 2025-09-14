use super::utils::Centered;
use dioxus::{html::textarea::placeholder, prelude::*};

#[component]
pub fn Login() -> Element {
    rsx! {
        Centered {
            background: "https://authserver.nju.edu.cn/authserver/custom/images/back3.jpg",

            form {
                id: "input-form",
                class: "center-children",
                method: "post",

                InputBox { name: "username", input_placeholder: "学号", input_type: "text" }
                InputBox { name: "password", input_placeholder: "密码", input_type: "password" }
                InputBox { name: "captcha", input_placeholder: "验证码", input_type: "text" }
            }
        }
    }
}

#[component]
fn InputBox(name: String, input_placeholder: String, input_type: String) -> Element {
    rsx! {
        input {
            id: name.clone(),
            class: "inputbox",
            name: name,
            placeholder: input_placeholder,
            type: input_type
        }
    }
}
