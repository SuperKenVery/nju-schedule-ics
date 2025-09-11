/// 与南大服务器对接
pub mod nju;

/// 对接各种学校的课表API
pub mod adapters;

/// 处理转换，从json到日历ics文件
pub mod schedule;

/// http服务器
pub mod server;
