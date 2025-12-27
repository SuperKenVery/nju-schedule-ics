use super::super::app::Route;
use super::super::utils::{ButtonWithLoading, Hero};
use dioxus::prelude::*;
use tracing::info;

#[component]
pub fn SchoolAPISelect() -> Element {
    let adapters = use_server_future(available_adapters)?;
    let active_idx = use_signal(|| 0);
    let mut wasm_loaded = use_signal(|| false);
    use_effect(move || {
        wasm_loaded.set(true);
    });

    rsx! {
        Hero {
            image: "https://authserver.nju.edu.cn/authserver/custom/images/back3.jpg",

            h1 {
                class: "mb-5 text-5xl font-bold text-neutral-content",
                "欢迎来到南哪另一课表"
            }
            p {
                class: "mb-5 text-neutral-content",
                "请选择你要用的接口："
            }

            match adapters() {
                None => rsx!{
                    p { "加载中……" }
                },
                Some(Err(error)) => rsx! {
                    p { "加载失败: {error:?}" }
                },
                Some(Ok(adapters)) => rsx!{
                    SchoolAdapterMenu { adapters: adapters.clone(), active_index: active_idx }
                }
            }

            ButtonWithLoading {
                class: "btn btn-primary",
                onclick:  move |_event| async move {
                    if let Some(Ok(adapters)) = adapters() &&
                        let Some(adapter_name) = adapters.get(active_idx()) {
                        set_school(adapter_name.to_string()).await?;
                        info!("Selecting school: {}", adapter_name);

                        let nav = navigator();
                        nav.push(Route::Login {  });
                    }
                    Ok(())
                },

                if wasm_loaded() {
                    "下一步"
                }else{
                    "加载中"
                    span { class: "loading loading-spinner" }
                }
            }
        }
    }
}

#[component]
fn SchoolAdapterMenu(adapters: Vec<String>, active_index: Signal<usize>) -> Element {
    rsx! {
        ul {
            class: "menu rainbow-shadow mx-auto mb-5 bg-base-200 w-96 text-black",

            for (idx, adapter) in adapters.iter().enumerate() {
                li {
                    onclick: move |_event| {
                        active_index.set(idx);
                    },

                    a {
                        class: String::from("text-center justify-center ") + if idx==active_index() { "bg-gray-300" } else {""},
                        // class: "text-center w-full",
                        {adapter.clone()}
                    }
                }
            }
        }
    }
}

#[cfg(feature = "server")]
use crate::{adapters::login_process::LoginProcess, server::state::ServerState};

/// Get available adapters, also getting a session ID.
#[get("/api/all_adapters", state: ServerState)]
pub async fn available_adapters() -> Result<Vec<String>, ServerFnError> {
    let school_adapters = state.school_adapters.lock().await;

    Ok(school_adapters
        .keys()
        .cloned()
        .map(|s| s.to_string())
        .collect())
}

#[post("/api/set_school", session: LoginProcess)]
pub async fn set_school(name: String) -> Result<()> {
    session.select_school(name).await?;

    Ok(())
}
