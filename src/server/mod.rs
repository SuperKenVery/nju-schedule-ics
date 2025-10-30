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

/// 在收到请求时，生成并返回ics日历文件
#[cfg(feature = "server")]
pub mod calendar;
