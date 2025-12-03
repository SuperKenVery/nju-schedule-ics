use super::super::utils::Hero;
use dioxus::prelude::*;
use urlencoding::encode as url_encode;

#[component]
pub fn ViewLink() -> Element {
    let prefix = use_server_future(get_subscription_link_prefix)?;
    let db_key = use_server_future(get_subscription_key)?;
    let school = use_server_future(get_selected_school_name)?;
    let subscription_url = match (prefix(), school(), db_key()) {
        (Some(Ok(prefix)), Some(Ok(adapter_api)), Some(Ok(key))) => Ok(format!(
            "{}/calendar/{}/{}/schedule.ics",
            prefix.replace("https", "webcal"),
            url_encode(adapter_api.as_str()),
            key
        )),
        _ => Err("登陆状态异常，无法获取订阅链接".to_string()),
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

                    match subscription_url.clone() {
                        Ok(url) => {
                            tracing::debug!("Showing subscription link");
                            rsx!{
                            Link {
                                class: "link-success break-all",
                                to: url.clone(),
                                { url.clone() }
                            }
                        }},
                        Err(msg) => {
                            rsx!{
                            Link {
                                class: "link-error break-all",
                                to: "",
                                { msg.clone() }
                            }
                        }}
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

#[cfg(feature = "server")]
use crate::{adapters::login_process::LoginProcess, server::state::ServerState};

/// Get the protocol and host part of subscription link,
/// without trailing slash.
#[get("/api/subscription_prefix", state: ServerState)]
async fn get_subscription_link_prefix() -> Result<String> {
    let site_url = &state.site_url;
    Ok(site_url.clone())
}

#[get("/api/subscription_key", session: LoginProcess)]
async fn get_subscription_key() -> Result<String> {
    Ok(session
        .cred_db_key()
        .await
        .context("Failed to get subscription link key")?)
}

#[get("/api/selected_school_name", session: LoginProcess)]
async fn get_selected_school_name() -> Result<String> {
    Ok(session
        .selected_school_adapter_name()
        .await
        .context("No selected school yet")?)
}
