//! 预编译的正则表达式模式
//!
//! 使用 `once_cell::Lazy` 避免重复编译正则表达式，提高性能。

use once_cell::sync::Lazy;
use regex::Regex;

// ============================================================================
// 版本号提取正则
// ============================================================================

/// 版本号提取正则表达式
///
/// 支持以下格式：
/// - `ver.1.0`, `ver 1.0`, `v.1.0`, `v 1.0`
/// - `_1.0`, `_1.0.0`
/// - `1.0`, `1.0.0` (结尾)
pub static VERSION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)ver\.?\s*(\d+(?:\.\d+)*)").unwrap(),
        Regex::new(r"(?i)v\.?\s*(\d+(?:\.\d+)*)").unwrap(),
        Regex::new(r"_(\d+\.\d+(?:\.\d+)*)").unwrap(),
        Regex::new(r"(\d+\.\d+(?:\.\d+)*)$").unwrap(),
    ]
});

// ============================================================================
// 搜索关键词提取正则
// ============================================================================

/// 前缀标签匹配正则（需要移除）
///
/// 匹配：`【标签】`, `[标签]`
pub static PREFIX_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"【[^】]*】").unwrap(),
        Regex::new(r"\[[^\]]*\]").unwrap(),
    ]
});

/// 版本号移除正则（支持字母后缀）
///
/// 匹配：`ver.1.0a`, `v1.0b`, `_1.0.0c`, `1.0d` (结尾)
pub static VERSION_REMOVAL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)ver\.?\s*\d+(?:\.\d+)*[a-z]*").unwrap(),
        Regex::new(r"(?i)v\.?\s*\d+(?:\.\d+)*[a-z]*").unwrap(),
        Regex::new(r"_\d+\.\d+(?:\.\d+)*[a-z]*").unwrap(),
        Regex::new(r"\d+\.\d+(?:\.\d+)*[a-z]*$").unwrap(),
    ]
});

/// 平台标识匹配正则（需要移除）
///
/// 匹配：`PC版`, `Windows版`, `Mac版`, `Linux版`, `Android版`, `iOS版`
pub static PLATFORM_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)PC版").unwrap(),
        Regex::new(r"(?i)Windows版?").unwrap(),
        Regex::new(r"(?i)Mac版?").unwrap(),
        Regex::new(r"(?i)Linux版?").unwrap(),
        Regex::new(r"(?i)Android版?").unwrap(),
        Regex::new(r"(?i)iOS版?").unwrap(),
    ]
});

/// 后缀标签匹配正则（需要移除）
///
/// 匹配：`AI汉化`, `汉化版`, `中文版`, `官中` (结尾)
pub static SUFFIX_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)AI汉化$").unwrap(),
        Regex::new(r"(?i)汉化版?$").unwrap(),
        Regex::new(r"(?i)中文版?$").unwrap(),
        Regex::new(r"(?i)官中$").unwrap(),
    ]
});

