use dioxus::prelude::*;

#[component]
pub fn Centered(background: Option<String>, children: Element) -> Element {
    rsx! {
        div {
            class: "center-children fullscreen",

            div {
                id: "input-container",
                class: "card center-children",

                {children}
            }
        }

        img {
            id: "background",
            src: background
        }
    }
}

#[derive(Clone)]
pub struct ClientState {
    pub session_id: Option<String>,
}
