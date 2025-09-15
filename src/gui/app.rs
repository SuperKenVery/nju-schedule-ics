use crate::gui::utils::ClientState;

use super::login::Login;
use super::select_school::SchoolAPISelect;
use dioxus::prelude::*;
use tracing::info;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND: Asset = asset!("/assets/tailwind_output.css");
const RAINBOW_SHADOW: Asset = asset!("/assets/rainbow_shadow.css");

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    SchoolAPISelect,
    #[route("/login")]
    Login {},
    // #[route("/view_link")]
    // SubscriptionLink,
}

#[component]
pub fn App() -> Element {
    let client_state = use_context_provider(|| Signal::new(ClientState { session_id: None }));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND }
        document::Link { rel: "stylesheet", href: RAINBOW_SHADOW }

        div {
            h2 { "Debug variable display" }
            p { "session id = {client_state().session_id:?}" }
        }
        Router::<Route> {}
    }
}
