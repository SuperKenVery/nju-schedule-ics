/// 对接各种学校的课表API
///
/// 要适配一个新学校，你需要新建一个struct，然后为其实现[`adapters::traits::School`]，
/// 最后在[`server::state::ServerState::from_config`]中把它加上。
#[cfg(feature = "server")]
pub mod adapters;

/// 使用dioxus构建的全栈应用，前端部分
pub mod gui;

/// http服务器
pub mod server;

/// 学校之外的逻辑处理，比如调休
#[cfg(feature = "server")]
pub mod plugins;
