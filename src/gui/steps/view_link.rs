use super::super::app::Route;
use super::super::utils::{ButtonWithLoading, ClientState, CustomError, Hero, Result, ResultExt};
use daisy_rsx::{Card, CardBody};
use dioxus::prelude::{
    server_fn::{ServerFn, error::NoCustomError},
    *,
};
use std::ops::Not;
use std::string;
use tracing::{debug, info};
use urlencoding::encode as url_encode;

#[component]
pub fn ViewLink() -> Element {
    let client_state = use_context::<Signal<ClientState>>();
    let prefix = use_server_future(get_subscription_link_prefix)?;
    let subscription_url = match (
        prefix(),
        client_state().school_adapter_api,
        client_state().db_key,
    ) {
        (Some(Ok(prefix)), Some(adapter_api), Some(key)) => {
            format!(
                "{}/{}/{}",
                prefix.replace("https", "webcal"),
                url_encode(adapter_api.as_str()),
                key
            )
        }
        _ => "登陆状态异常，无法获取订阅链接".to_string(),
    };

    rsx! {
        Hero {
            image: "https://authserver.nju.edu.cn/authserver/custom/images/back3.jpg",

            div {
                class: "card bg-base-200 w-96 card-xl shadow-sm",

                div {
                    class: "card-body",

                    h2 { class: "card-title", "订阅成功" }
                    p { "您的订阅链接为：" }
                    Link {
                        class: "link-success",
                        to: subscription_url.clone(),
                        { subscription_url.clone() }
                    }

                    Howto {
                        title: "苹果平台（iOS/macOS）",
                        p {"直接点击此链接，就会跳转到系统日历app的导入界面，确认导入即可。"}
                    }
                    Howto {
                        title: "安卓设备",
                        p  {
                            "请先下载"
                            a {
                                class: "link link-accent",
                                href: "https://mirror.nju.edu.cn/fdroid/repo/at.bitfire.icsdroid_89.apk",
                                "ICSx⁵"
                            }
                            "，然后复制本链接，在ICSx⁵软件中添加订阅。"
                        }
                    }
                    Howto {
                        title: "Windows",
                        p { "可以使用outlook中的日历订阅能力，或者使用你自己喜欢的日历app" }
                    }
                    Howto {
                        title: "Linux",
                        p { "GNOME自带的日历就可以添加订阅" }
                    }
                }
            }
        }
    }
}

#[component]
fn Howto(title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "collapse collapse-arrow bg-base-100 border-base-300 border",

            input { type: "checkbox" }
            div { class: "collapse-title font-semibold", {title} }
            div { class: "collapse-content text-sm", {children} }
        }
    }
}

/// Get the protocol and host part of subscription link,
/// without trailing slash.
#[server]
async fn get_subscription_link_prefix() -> Result<String, ServerFnError<CustomError>> {
    use crate::server::state::ServerState;

    let FromContext(state): FromContext<ServerState> = extract().await.to_sfn()?;
    let site_url = &state.site_url;
    Ok(site_url.clone())
}
