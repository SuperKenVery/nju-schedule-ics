#![recursion_limit = "512"]

/// 对接各种学校的课表API
pub mod adapters;

/// 使用dioxus构建的全栈应用，包括网页前端和服务器后端。
pub mod gui;

/// http服务器
pub mod server;
