use std::fmt;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
    Debug,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "ℹ️ "),
            LogLevel::Success => write!(f, "✅"),
            LogLevel::Warning => write!(f, "⚠️ "),
            LogLevel::Error => write!(f, "❌"),
            LogLevel::Debug => write!(f, "🔍"),
        }
    }
}

/// 日志事件
#[derive(Debug, Clone)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,
    pub details: Option<String>,
}

impl LogEvent {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// 格式化为单行输出
    pub fn format_compact(&self) -> String {
        if let Some(details) = &self.details {
            format!("{} {} - {}", self.level, self.message, details)
        } else {
            format!("{} {}", self.level, self.message)
        }
    }

    /// 格式化为多行输出
    pub fn format_detailed(&self) -> String {
        let mut output = format!("{} {}", self.level, self.message);
        if let Some(details) = &self.details {
            output.push_str(&format!("\n   {}", details));
        }
        output
    }
}

/// 扫描进度事件
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub current_item: String,
}

impl ScanProgress {
    pub fn new(current: usize, total: usize, current_item: impl Into<String>) -> Self {
        Self {
            current,
            total,
            current_item: current_item.into(),
        }
    }

    pub fn format(&self) -> String {
        format!(
            "[{}/{}] 正在处理: {}",
            self.current, self.total, self.current_item
        )
    }

    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f32 / self.total as f32) * 100.0
        }
    }
}

/// 查询结果摘要
#[derive(Debug, Clone)]
pub struct QuerySummary {
    pub query: String,
    pub total_results: usize,
    pub provider_results: Vec<(String, usize)>,
    pub duration_ms: u64,
}

impl QuerySummary {
    pub fn format_compact(&self) -> String {
        format!(
            "查询 \"{}\" 完成: {} 条结果 ({}ms)",
            self.query, self.total_results, self.duration_ms
        )
    }

    pub fn format_detailed(&self) -> String {
        let mut output = format!(
            "📊 查询完成: \"{}\" ({}ms)\n",
            self.query, self.duration_ms
        );
        output.push_str(&format!("   总结果: {} 条\n", self.total_results));
        
        if !self.provider_results.is_empty() {
            output.push_str("   来源分布:\n");
            for (provider, count) in &self.provider_results {
                output.push_str(&format!("     - {}: {} 条\n", provider, count));
            }
        }
        
        output
    }
}

/// 简化的日志记录器
pub struct SimpleLogger {
    verbose: bool,
}

impl SimpleLogger {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn log(&self, event: &LogEvent) {
        if self.verbose {
            println!("{}", event.format_detailed());
        } else {
            println!("{}", event.format_compact());
        }
    }

    pub fn progress(&self, progress: &ScanProgress) {
        if self.verbose {
            println!("{}", progress.format());
        }
    }

    pub fn summary(&self, summary: &QuerySummary) {
        if self.verbose {
            print!("{}", summary.format_detailed());
        } else {
            println!("{}", summary.format_compact());
        }
    }

    pub fn section(&self, title: &str) {
        if self.verbose {
            println!("\n{}", "=".repeat(80));
            println!("🎯 {}", title);
            println!("{}", "=".repeat(80));
        }
    }

    pub fn subsection(&self, title: &str) {
        if self.verbose {
            println!("\n{}", "-".repeat(60));
            println!("  {}", title);
            println!("{}", "-".repeat(60));
        }
    }
}

use std::sync::OnceLock;

/// 全局日志记录器实例
static LOGGER: OnceLock<SimpleLogger> = OnceLock::new();

pub fn init_logger(verbose: bool) {
    let _ = LOGGER.set(SimpleLogger::new(verbose));
}

pub fn get_logger() -> &'static SimpleLogger {
    LOGGER.get_or_init(|| SimpleLogger::new(true))
}

// 便捷宏
#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        $crate::logger::get_logger().log(&$crate::logger::LogEvent::new(
            $crate::logger::LogLevel::Info,
            $msg,
        ));
    };
    ($msg:expr, $details:expr) => {
        $crate::logger::get_logger().log(
            &$crate::logger::LogEvent::new($crate::logger::LogLevel::Info, $msg)
                .with_details($details),
        );
    };
}

#[macro_export]
macro_rules! log_success {
    ($msg:expr) => {
        $crate::logger::get_logger().log(&$crate::logger::LogEvent::new(
            $crate::logger::LogLevel::Success,
            $msg,
        ));
    };
    ($msg:expr, $details:expr) => {
        $crate::logger::get_logger().log(
            &$crate::logger::LogEvent::new($crate::logger::LogLevel::Success, $msg)
                .with_details($details),
        );
    };
}

#[macro_export]
macro_rules! log_warning {
    ($msg:expr) => {
        $crate::logger::get_logger().log(&$crate::logger::LogEvent::new(
            $crate::logger::LogLevel::Warning,
            $msg,
        ));
    };
    ($msg:expr, $details:expr) => {
        $crate::logger::get_logger().log(
            &$crate::logger::LogEvent::new($crate::logger::LogLevel::Warning, $msg)
                .with_details($details),
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        $crate::logger::get_logger().log(&$crate::logger::LogEvent::new(
            $crate::logger::LogLevel::Error,
            $msg,
        ));
    };
    ($msg:expr, $details:expr) => {
        $crate::logger::get_logger().log(
            &$crate::logger::LogEvent::new($crate::logger::LogLevel::Error, $msg)
                .with_details($details),
        );
    };
}

#[macro_export]
macro_rules! log_debug {
    ($msg:expr) => {
        $crate::logger::get_logger().log(&$crate::logger::LogEvent::new(
            $crate::logger::LogLevel::Debug,
            $msg,
        ));
    };
    ($msg:expr, $details:expr) => {
        $crate::logger::get_logger().log(
            &$crate::logger::LogEvent::new($crate::logger::LogLevel::Debug, $msg)
                .with_details($details),
        );
    };
}

