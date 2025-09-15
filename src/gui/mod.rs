pub mod app;
mod login;
mod select_school;
pub mod utils;

pub fn start_app() {
    dioxus::launch(app::App);
}
