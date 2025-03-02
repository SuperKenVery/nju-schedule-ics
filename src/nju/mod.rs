//! nju：与学校服务器对接的部分

/// 从南大服务器获取课程信息
pub mod getcourse;

/// 登陆到南京大学统一认证，获取cookie
pub mod login;

/// 解析服务器返回的json课程信息，转换为Course结构体
pub mod parse_course;
