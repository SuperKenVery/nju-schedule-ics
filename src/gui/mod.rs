pub mod app;

pub fn start_app() {
    dioxus::launch(app::App);
}
