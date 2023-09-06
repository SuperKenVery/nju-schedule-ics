/* 课程相关数据结构
 * 存储名称，地点，上课时间
 * 以及从json数据解析
 */
pub mod course;
/* 从南大服务器获取课程信息 */
mod getcourse;
/* 时间相关封装 */
pub mod time;
/* 从Course生成ics文件 */
pub mod calendar;
/* 根据位置名称生成经纬度 */
mod location;
