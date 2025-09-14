pub mod app;
mod login;
mod select_school;
mod utils;

pub fn start_app() {
    dioxus::launch(app::App);
}
