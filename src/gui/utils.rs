use anyhow::anyhow;
use daisy_rsx::marketing::customer_logos::Customers;
use dioxus::prelude::{server_fn::error::NoCustomError, *};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use tracing::error;

/// An element that centers its children
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

/// The global app state for web page.
#[derive(Clone)]
pub struct ClientState {
    pub session_id: Option<String>,
}

/// # A custom error for server_fn
/// Mostly used in client side.
///
/// # Why?
/// In order to use `?` on a ServerFnError in button event handlers,
/// the custom error has to implement std::error::Error, so the default
/// NoCustonError cannot be used. In order to use it inside ServerFnError,
/// it has to implement FromStr so anyhow::Error isn't ok.
#[derive(Debug, Clone)]
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
