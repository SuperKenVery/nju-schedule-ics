use super::app::Route;
use super::utils::Result;
use crate::gui::utils::{ClientState, CustomError, ResultExt};
use dioxus::prelude::{
    server_fn::{ServerFn, error::NoCustomError},
    *,
};
use tracing::{debug, info};

#[component]
pub fn SchoolAPISelect() -> Element {
    let mut client_state = use_context::<Signal<ClientState>>();

    let adapters = use_server_future(available_adapters)?;
    let session_id = use_resource(move || async move {
        let session_id = get_session().await;
        client_state.write().session_id = session_id.clone().ok();
        session_id
    });
    let active_idx = use_signal(|| 0);

    rsx! {
        Hero {
            image: "https://authserver.nju.edu.cn/authserver/custom/images/back3.jpg",

            h1 {
                class: "mb-5 text-5xl font-bold",
                "欢迎来到南哪另一课表"
            }
            p {
                class: "mb-5",
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

            if let Some(Ok(_session)) = session_id() {} else {
                p { "无法获取会话" }
            }

            button {
                class: "btn btn-primary",
                onclick:  move |event| async move {
                    if let Some(Ok(adapters)) = adapters() &&
                        let Some(adapter_name) = adapters.get(active_idx()) &&
                        let Some(Ok(session_id)) = session_id(){
                        set_school(adapter_name.to_string(), session_id).await?;
                        info!("Selecting school: {}", adapter_name);

                        let nav = navigator();
                        nav.push(Route::Login {  });
                    }
                    Ok(())
                },
                "下一步"
            }
        }
    }
}

#[component]
fn Hero(image: Option<String>, children: Element) -> Element {
    rsx! {
        div {
            class: "hero min-h-screen",
            style: if let Some(image)=image { "background-image: url({image})" } else { "" },

            div { class: "hero-overlay" }
            div {
                class: "hero-content text-neutral-content text-center",

                div {
                    // class: "max-w-md",
                    {children}
                }
            }
        }
    }
}

#[component]
fn SchoolAdapterMenu(adapters: Vec<String>, active_index: Signal<usize>) -> Element {
    rsx! {
        ul {
            class: "menu rainbow-shadow mx-auto mb-5 bg-base-200 w-56 text-black",

            for (idx, adapter) in adapters.iter().enumerate() {
                li {
                    onclick: move |_event| {
                        active_index.set(idx);
                    },

                    a {
                        class: if idx==active_index() { "menu-active" } else {""},
                        {adapter.clone()}
                    }
                }
            }
        }
    }
}

/// Get available adapters, also getting a session ID.
#[server]
pub async fn available_adapters() -> Result<Vec<String>, ServerFnError> {
    use crate::adapters::all_school_adapters::school_adapters;

    Ok(school_adapters().iter().map(|x| x.to_string()).collect())
}

/// Get a new UnfinishedLoginSession
#[server]
pub async fn get_session() -> Result<String, ServerFnError<CustomError>> {
    use crate::server::state::{ServerState, UnfinishedLoginSession};
    use uuid::Uuid;

    let FromContext(state): FromContext<ServerState> = extract().await.to_sfn()?;
    let mut session_id = Uuid::new_v4().to_string();
    let mut sessions = state.unfinished_login_sessions.lock().await;
    while sessions.contains_key(&session_id) {
        // UUID collision
        session_id = Uuid::new_v4().to_string();
    }

    sessions.insert(session_id.clone(), UnfinishedLoginSession::Started);

    Ok(session_id)
}

#[server]
pub async fn set_school(
    name: String,
    session_id: String,
) -> Result<(), ServerFnError<CustomError>> {
    use crate::server::state::{ServerState, UnfinishedLoginSession};

    let FromContext(state): FromContext<ServerState> = extract().await.to_sfn()?;
    let mut sessions = state.unfinished_login_sessions.lock().await;
    let session = sessions
        .get_mut(&session_id)
        .context("Session not found")
        .to_sfn()?;

    session.select_school(name).await.to_sfn()?;

    Ok(())
}
