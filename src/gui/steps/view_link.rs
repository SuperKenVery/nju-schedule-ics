use super::super::app::Route;
use super::super::utils::{ButtonWithLoading, ClientState, CustomError, Hero, Result, ResultExt};
use daisy_rsx::{Card, CardBody};
use dioxus::prelude::{
    server_fn::{ServerFn, error::NoCustomError},
    *,
};
use std::ops::Not;
use tracing::{debug, info};

#[component]
pub fn ViewLink() -> Element {
    rsx! {
        Hero {
            image: "https://authserver.nju.edu.cn/authserver/custom/images/back3.jpg",

            Card {
                class: "card-border bg-base-100 w-96",

                CardBody {
                    h1 { "订阅成功" }
                    p { "您的订阅链接为：" }
                }
            }
        }
    }
}
