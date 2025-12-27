pub mod app;
mod steps;
pub mod utils;
use dioxus::prelude::dioxus_fullstack;

pub fn start_app() {
    let server_path = dioxus_fullstack::get_server_url();
    tracing::info!("Server path: {}", server_path);
    dioxus::launch(app::App);
}
