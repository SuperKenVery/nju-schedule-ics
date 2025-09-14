use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, sync::LazyLock};

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
