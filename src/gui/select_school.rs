use dioxus::prelude::*;
use tracing::debug;

#[component]
pub fn SchoolAPISelect() -> Element {
    let adapters =
        use_resource(
            move || async move { crate::server::apis::adapters::available_adapters().await },
        );

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

            match &*adapters.read() {
                Some(Err(error)) => rsx! {
                    p { "无法加载所有接口：{error}" }
                },
                None => rsx!{
                    p { "加载中……" }
                },
                Some(Ok(adapters)) => rsx!{
                    SchoolAdapterMenu { adapters: adapters.clone() }
                }
            }

            button {
                class: "btn btn-primary",
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
fn SchoolAdapterMenu(adapters: Vec<String>) -> Element {
    let mut active = use_signal(|| 0);

    rsx! {
        ul {
            class: "menu rainbow-shadow mx-auto mb-5 bg-base-200 w-56 text-black",

            for (idx, adapter) in adapters.iter().enumerate() {
                li {
                    onclick: move |_event| {
                        active.set(idx);
                    },

                    a {
                        class: if idx==active() { "menu-active" } else {""},
                        {adapter.clone()}
                    }
                }
            }
        }
    }
}
