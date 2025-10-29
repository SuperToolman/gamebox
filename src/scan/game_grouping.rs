//! 游戏路径分组相关的结构体和算法
//!
//! 该模块负责将扫描到的游戏文件路径按照游戏根目录进行分组，
//! 并提取游戏的版本号和搜索关键词。

use ignore::DirEntry;
use serde::{Deserialize, Serialize};
use crate::scan::utils::{extract_search_key, extract_version, find_common_parent_dir};

/// 路径分组结果
///
/// 表示一个游戏的根目录和其下的所有可执行文件路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathGroupResult {
    /// 游戏根目录的完整路径
    pub root_path: String,
    /// 游戏根目录的名称（最后一个路径组件）
    pub child_root_name: String,
    /// 相对于游戏根目录的子路径列表（可执行文件）
    pub child_path: Vec<String>,
    /// 用于搜索的关键词（去除前缀标签和版本号）
    pub search_key: String,
    /// 从目录名中提取的版本号
    pub version: Option<String>,
}

/// 目录条目过滤器 trait
///
/// 用于过滤和处理目录条目
pub trait DirEntryFilter {
    /// 过滤父目录名称
    ///
    /// 该方法用于过滤掉不需要的目录条目。
    /// 目前的实现返回所有条目（不进行过滤）。
    fn filter_parent_directory_names(&self) -> Vec<DirEntry>;
}

impl DirEntryFilter for Vec<DirEntry> {
    fn filter_parent_directory_names(&self) -> Vec<DirEntry> {
        // 目前不进行过滤，返回所有条目
        // 未来可以在这里添加过滤逻辑，例如：
        // - 过滤掉隐藏目录
        // - 过滤掉系统目录
        // - 过滤掉特定模式的目录
        self.clone()
    }
}

/// 基于最近公共父目录分组
///
/// 将多个 exe 文件路径按照它们的最近公共父目录分组。
/// 每组的游戏根目录是该组所有 exe 文件的最近公共父目录。
///
/// # 参数
/// - `paths`: 扫描到的目录条目列表（通常是可执行文件）
///
/// # 返回
/// 分组后的路径结果列表
///
/// # 算法说明
/// 1. 找到所有路径的全局共同前缀（扫描根目录）
/// 2. 按照扫描根目录后的第一级目录进行初步分组
/// 3. 对每个第一级分组，找到最近公共父目录
/// 4. 使用启发式规则决定游戏根目录：
///    - 默认使用第一级目录
///    - 如果第一级包含前缀标签（如【RPG】），且第二级不是平台名称，则使用第二级
/// 5. 提取版本号和搜索关键词
pub fn paths_group(paths: Vec<DirEntry>) -> Vec<PathGroupResult> {
    if paths.is_empty() {
        return Vec::new();
    }

    // 优化：直接处理路径，减少字符串分配
    // 将路径分割为组件，只在需要时进行字符串分配
    let path_components: Vec<Vec<String>> = paths
        .iter()
        .map(|entry| {
            let path_str = entry.path().to_string_lossy();

            // 只在包含反斜杠时才进行替换（Windows 路径）
            if path_str.contains('\\') {
                // 使用 replace 一次性替换所有反斜杠
                path_str
                    .replace('\\', "/")
                    .split('/')
                    .map(|s| s.to_string())
                    .collect()
            } else {
                // Unix 路径，直接分割
                path_str.split('/').map(|s| s.to_string()).collect()
            }
        })
        .collect();

    // 找到所有路径的全局共同前缀（扫描根目录）
    let mut scan_root_len = 0;
    if !path_components.is_empty() {
        let first_path = &path_components[0];
        'outer: for i in 0..first_path.len() {
            let component = &first_path[i];
            for path in &path_components {
                if i >= path.len() || &path[i] != component {
                    break 'outer;
                }
            }
            scan_root_len = i + 1;
        }
    }

    // 按照扫描根目录后的第一级目录进行初步分组
    let mut first_level_groups: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();

    for (idx, path) in path_components.iter().enumerate() {
        if scan_root_len < path.len() {
            let first_level_dir = path[scan_root_len].clone();
            first_level_groups
                .entry(first_level_dir)
                .or_insert_with(Vec::new)
                .push(idx);
        }
    }

    // 对每个第一级分组，找到最近公共父目录
    let mut results: Vec<PathGroupResult> = Vec::new();

    for (_first_level_dir, indices) in first_level_groups {
        // 获取这个组的所有路径
        let group_paths: Vec<Vec<String>> = indices
            .iter()
            .map(|&idx| path_components[idx].clone())
            .collect();

        // 找到这组路径的最近公共父目录
        let common_parent_len = find_common_parent_dir(&group_paths);

        // 决定游戏根目录：
        // 默认使用第一级目录（scan_root_len + 1）
        let mut game_root_len = scan_root_len + 1;

        // 如果公共父目录是第二级（scan_root_len + 2），需要判断是否使用第二级
        if common_parent_len == scan_root_len + 2
            && common_parent_len <= path_components[indices[0]].len()
        {
            let first_level_name = &path_components[indices[0]][scan_root_len];
            let second_level_name = &path_components[indices[0]][scan_root_len + 1];

            // 启发式规则：
            // 1. 如果第二级目录名是通用的平台名称（Windows, Linux, Mac等），使用第一级
            // 2. 否则，如果第一级包含前缀标签，使用第二级
            let common_platform_names = ["Windows", "Linux", "Mac", "MacOS", "Android", "iOS"];
            let is_platform_dir = common_platform_names
                .iter()
                .any(|&name| second_level_name == name);

            if !is_platform_dir {
                let first_has_prefix =
                    first_level_name.contains('【') || first_level_name.contains('[');

                if first_has_prefix {
                    // 使用第二级作为游戏根目录
                    game_root_len = scan_root_len + 2;
                }
            }
        }

        // 构建游戏根目录路径
        let game_root_path =
            if game_root_len > 0 && game_root_len <= path_components[indices[0]].len() {
                path_components[indices[0]][0..game_root_len].join("/")
            } else {
                String::new()
            };

        // 提取游戏根目录名称（最后一个组件）
        let game_root_name =
            if game_root_len > 0 && game_root_len <= path_components[indices[0]].len() {
                path_components[indices[0]][game_root_len - 1].clone()
            } else {
                "Unknown".to_string()
            };

        // 构建相对路径列表（相对于游戏根目录）
        let mut child_paths: Vec<String> = Vec::new();
        for &idx in &indices {
            if game_root_len < path_components[idx].len() {
                let relative_path = path_components[idx][game_root_len..].join("/");
                child_paths.push(relative_path);
            }
        }

        // 提取版本号和搜索关键词
        let version = extract_version(&game_root_name);
        let search_key = extract_search_key(&game_root_name);

        results.push(PathGroupResult {
            root_path: game_root_path,
            child_root_name: game_root_name,
            child_path: child_paths,
            search_key,
            version,
        });
    }

    // 按照 child_path 的第一个元素排序，保证结果的一致性
    results.sort_by(|a, b| a.child_path.first().cmp(&b.child_path.first()));

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_group_result_serialization() {
        let result = PathGroupResult {
            root_path: "C:/Games/Game1".to_string(),
            child_root_name: "Game1".to_string(),
            child_path: vec!["game.exe".to_string()],
            search_key: "Game1".to_string(),
            version: Some("1.0".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: PathGroupResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.root_path, deserialized.root_path);
        assert_eq!(result.child_root_name, deserialized.child_root_name);
        assert_eq!(result.search_key, deserialized.search_key);
        assert_eq!(result.version, deserialized.version);
    }
}

