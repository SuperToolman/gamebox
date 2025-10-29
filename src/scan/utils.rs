//! 扫描相关的工具函数

use std::path::PathBuf;
use crate::scan::patterns::{
    VERSION_PATTERNS, PREFIX_PATTERNS, VERSION_REMOVAL_PATTERNS,
    PLATFORM_PATTERNS, SUFFIX_PATTERNS,
};

/// 计算目录大小（异步版本，使用迭代而非递归避免栈溢出）
///
/// # 参数
/// - `dir_path`: 要计算大小的目录路径
///
/// # 返回
/// 目录的总大小（字节）
pub async fn calculate_directory_size_async(dir_path: PathBuf) -> u64 {
    use tokio::fs;

    let mut total_size = 0u64;
    let mut stack = vec![dir_path];

    while let Some(path) = stack.pop() {
        match fs::read_dir(&path).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    match entry.metadata().await {
                        Ok(metadata) => {
                            if metadata.is_file() {
                                total_size += metadata.len();
                            } else if metadata.is_dir() {
                                stack.push(entry.path());
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
            Err(_) => continue,
        }
    }

    total_size
}

/// 从游戏目录名中提取版本号
///
/// 支持以下格式：
/// - `ver.1.0`, `ver 1.0`, `v.1.0`, `v 1.0`
/// - `_1.0`, `_1.0.0`
/// - `1.0`, `1.0.0` (结尾)
///
/// # 参数
/// - `dir_name`: 目录名称
///
/// # 返回
/// 提取到的版本号，如果没有找到则返回 `None`
pub fn extract_version(dir_name: &str) -> Option<String> {
    // 使用预编译的正则表达式（避免重复编译）
    for re in VERSION_PATTERNS.iter() {
        if let Some(captures) = re.captures(dir_name) {
            if let Some(version) = captures.get(1) {
                return Some(version.as_str().to_string());
            }
        }
    }

    None
}

/// 从游戏目录名中提取搜索关键词
///
/// 去除常见的前缀标签和版本号，如：【RPG官中】、【SLG汉化】、v1.0 等
///
/// # 参数
/// - `dir_name`: 目录名称
///
/// # 返回
/// 清理后的搜索关键词
///
/// # 示例
/// ```
/// use scanners::scan::extract_search_key;
///
/// let key = extract_search_key("【RPG官中】游戏名称 v1.0");
/// assert_eq!(key, "游戏名称");
/// ```
pub fn extract_search_key(dir_name: &str) -> String {
    let mut result = dir_name.to_string();

    // 1. 移除前缀标签（使用预编译的正则表达式）
    for re in PREFIX_PATTERNS.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    // 2. 移除版本号（使用预编译的正则表达式）
    for re in VERSION_REMOVAL_PATTERNS.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    // 3. 移除平台标识（使用预编译的正则表达式）
    for re in PLATFORM_PATTERNS.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    // 4. 移除常见的后缀（使用预编译的正则表达式）
    for re in SUFFIX_PATTERNS.iter() {
        result = re.replace_all(&result, "").to_string();
    }

    // 5. 清理多余的空白和特殊字符
    result = result.trim().to_string();

    // 移除末尾的下划线、空格、点号、波浪号
    while result.ends_with('_') || result.ends_with(' ') || result.ends_with('.') || result.ends_with('~') {
        result.pop();
    }

    result = result.trim().to_string();

    // 如果结果为空，返回原始名称
    if result.is_empty() {
        dir_name.to_string()
    } else {
        result
    }
}

/// 找到一组路径的最近公共父目录（不包括文件名）
///
/// # 参数
/// - `paths`: 路径组件的向量列表
///
/// # 返回
/// 公共父目录的长度（组件数量）
///
/// # 示例
/// ```
/// use scanners::scan::find_common_parent_dir;
///
/// let paths = vec![
///     vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "game.exe".to_string()],
///     vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "data".to_string(), "game.exe".to_string()],
/// ];
/// let common_len = find_common_parent_dir(&paths);
/// assert_eq!(common_len, 3); // C:\Games\Game1
/// ```
pub fn find_common_parent_dir(paths: &[Vec<String>]) -> usize {
    if paths.is_empty() {
        return 0;
    }

    // 找到最短路径的长度（排除文件名，所以 -1）
    let min_len = paths.iter().map(|p| p.len().saturating_sub(1)).min().unwrap_or(0);

    let mut common_len = 0;
    for i in 0..min_len {
        let component = &paths[0][i];

        // 检查所有路径在这个位置是否都有相同的组件
        if paths.iter().all(|p| i < p.len() && &p[i] == component) {
            common_len = i + 1;
        } else {
            break;
        }
    }

    common_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version() {
        assert_eq!(extract_version("Game v1.0"), Some("1.0".to_string()));
        assert_eq!(extract_version("Game ver.2.1.3"), Some("2.1.3".to_string()));
        assert_eq!(extract_version("Game_1.5"), Some("1.5".to_string()));
        assert_eq!(extract_version("Game 1.0.0"), Some("1.0.0".to_string()));
        assert_eq!(extract_version("Game"), None);
    }

    #[test]
    fn test_extract_search_key() {
        assert_eq!(extract_search_key("【RPG官中】游戏名称 v1.0"), "游戏名称");
        assert_eq!(extract_search_key("[SLG汉化]游戏名称"), "游戏名称");
        assert_eq!(extract_search_key("游戏名称 PC版"), "游戏名称");
        assert_eq!(extract_search_key("游戏名称 汉化版"), "游戏名称");
    }

    #[test]
    fn test_find_common_parent_dir() {
        let paths = vec![
            vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "game.exe".to_string()],
            vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "data".to_string(), "game.exe".to_string()],
        ];
        assert_eq!(find_common_parent_dir(&paths), 3);

        let paths = vec![
            vec!["C:".to_string(), "Games".to_string(), "Game1".to_string(), "game.exe".to_string()],
            vec!["C:".to_string(), "Games".to_string(), "Game2".to_string(), "game.exe".to_string()],
        ];
        assert_eq!(find_common_parent_dir(&paths), 2);
    }
}

