use anyhow::anyhow;
use dioxus::prelude::*;
use js_sys::{Array, Uint8Array};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use web_sys::{Blob, Url};

// === Utility Components ===

/// An element that centers its children
///
/// background: The background image URL.
/// card_bg: Whether to draw a card-like background, which has no border but a shadow. Default: true.
#[component]
pub fn Centered(background: Option<String>, card_bg: Option<bool>, children: Element) -> Element {
    rsx! {
        div {
            class: "center-children fullscreen",

            div {
                id: if card_bg==Some(false) { "" } else { "input-container" },
                class: if card_bg==Some(false){ "center-children" } else {"card center-children"},

                {children}
            }
        }

        img {
            id: "background",
            src: background
        }
    }
}

/// A Hero component with background image support.
///
/// See https://daisyui.com/components/hero/
#[component]
pub(super) fn Hero(image: Option<String>, children: Element) -> Element {
    rsx! {
        div {
            class: "hero min-h-screen",
            style: if let Some(image)=image { "background-image: url({image})" } else { "" },

            div { class: "hero-overlay" }
            div {
                // Remove text-neutral-content because that makes all text white
                class: "hero-content text-center",

                div {
                    // class: "max-w-md",
                    {children}
                }
            }
        }
    }
}

/// A button that adds a spinner and disables itself after being clicked
#[component]
pub fn ButtonWithLoading(
    class: String,
    onclick: EventHandler<MouseEvent>,
    r#type: Option<String>,
    children: Element,
) -> Element {
    let mut clicked = use_signal(|| false);

    rsx! {
        button {
            class: class,
            onclick: move |event| async move {
                clicked.set(true);
                onclick(event);
            },
            type: r#type,
            disabled: clicked(),

            {children}
            if clicked() {
                span { class: "loading loading-spinner" }
            }
        }
    }
}

// === JS Utils ===

pub fn to_blob_url(data: &[u8]) -> anyhow::Result<String> {
    let u8_array = Uint8Array::from(&data[..]);
    let array = Array::new();
    array.push(&u8_array);
    let blob = Blob::new_with_u8_array_sequence(&array).map_err(|e| anyhow!("{:?}", e))?;
    let url = Url::create_object_url_with_blob(&blob).map_err(|e| anyhow!("{:?}", e))?;

    Ok(url)
}
