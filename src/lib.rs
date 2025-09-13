/// 对接各种学校的课表API
#[cfg(feature = "server")]
pub mod adapters;

/// 使用dioxus构建的全栈应用，前端部分
pub mod gui;

/// http服务器
pub mod server;
