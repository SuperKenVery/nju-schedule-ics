/// HTTP服务器
#[cfg(feature = "server")]
pub mod server;

/// 解析命令行参数
#[cfg(feature = "server")]
pub mod config;

/// 让handler中可以使用`?`来处理错误
#[cfg(feature = "server")]
pub mod error;

/// 定义服务端的状态
#[cfg(feature = "server")]
pub mod state;
