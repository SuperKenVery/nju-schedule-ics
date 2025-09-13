use colog::format::CologStyle;
use colored::Colorize;
use log::{Level, Record};

pub struct LogTimePrefix;

impl CologStyle for LogTimePrefix {
    fn prefix_token(&self, level: &Level) -> String {
        format!(
            "[{}] {}",
            chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .blue()
                .bold(),
            self.level_color(level, self.level_token(level))
        )
    }

    // Show module name in log
    // fn format(&self, buf: &mut Formatter, record: &Record<'_>) -> Result<(), std::io::Error> {
    //     write!(
    //         buf,
    //         "{} [{}] {} - {}\n",
    //         self.prefix_token(&record.level()),
    //         record.module_path().unwrap_or("unknown"),
    //         record.level(),
    //         record.args()
    //     )
    // }
}
