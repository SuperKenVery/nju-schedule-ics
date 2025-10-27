use anyhow::anyhow;
use daisy_rsx::marketing::customer_logos::Customers;
use dioxus::prelude::{server_fn::error::NoCustomError, *};
use js_sys::{Array, Uint8Array};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use tracing::error;
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

// === Client State ===

/// The global app state for web page.
#[derive(Clone, Default)]
pub struct ClientState {
    /// The session_id for this session.
    /// Points to an UnfinishedLoginSession in server memory.
    pub session_id: Option<String>,
    /// The key for login credentials.
    /// Points to a record in server database. Only exists after logging in.
    pub db_key: Option<String>,
    /// The school adapter api selected for this session
    pub school_adapter_api: Option<String>,
}

// === Custom Error and Some extensions on Result ===

/// # A custom error for server_fn
/// Mostly used in client side.
///
/// # Why?
/// In order to use `?` on a [`ServerFnError`] in button event handlers,
/// the custom error has to implement [ `std::error::Error`], so the default
/// [`NoCustomError`] cannot be used. In order to use it inside ServerFnError,
/// it has to implement [`FromStr`] so [`anyhow::Error`] isn't ok.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomError {
    inner: String,
}

impl std::error::Error for CustomError {}
impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.as_str())
    }
}
impl FromStr for CustomError {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self {
            inner: s.to_string(),
        })
    }
}

pub trait ResultExt<T, E: Debug> {
    fn log_err(&self) -> ();

    /// Converts any [`Result<T, E>`] into [`Result<T, ServerFnError>`].
    fn to_sfn(self) -> Result<T>;
}

impl<T, E: Debug> ResultExt<T, E> for Result<T, E> {
    #[track_caller]
    fn log_err(&self) -> () {
        match self {
            Ok(_) => (),
            Err(e) => error!("{:?}", e),
        };
    }

    #[track_caller]
    fn to_sfn(self) -> Result<T> {
        match self {
            Ok(x) => Ok(x),
            Err(err) => Err(ServerFnError::WrappedServerError(CustomError {
                inner: format!("{:?}", err),
            })),
        }
    }
}

pub type Result<T, E = ServerFnError<CustomError>> = std::result::Result<T, E>;

// === JS Utils ===

pub fn to_blob_url(data: &[u8]) -> anyhow::Result<String> {
    let u8_array = Uint8Array::from(&data[..]);
    let array = Array::new();
    array.push(&u8_array);
    let blob = Blob::new_with_u8_array_sequence(&array).map_err(|e| anyhow!("{:?}", e))?;
    let url = Url::create_object_url_with_blob(&blob).map_err(|e| anyhow!("{:?}", e))?;

    Ok(url)
}
