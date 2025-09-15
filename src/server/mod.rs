/// HTTP服务器
#[cfg(feature = "server")]
pub mod server;

/// 解析命令行参数
#[cfg(feature = "server")]
pub mod config;

/// 让handler中可以使用`?`来处理错误
#[cfg(feature = "server")]
pub mod error;

/// 给客户端调用的接口实现
pub mod apis;
