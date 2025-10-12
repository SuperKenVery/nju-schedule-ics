pub mod app;
mod steps;
pub mod utils;

pub fn start_app() {
    dioxus::launch(app::App);
}
