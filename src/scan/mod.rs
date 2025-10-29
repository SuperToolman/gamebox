//! 游戏扫描模块
//!
//! 该模块提供了游戏文件扫描和元数据获取的功能。
//!
//! # 主要组件
//!
//! - [`GameScanner`] - 游戏扫描器，用于扫描本地游戏文件并获取元数据
//! - [`PathGroupResult`] - 路径分组结果
//! - 工具函数 - 版本提取、搜索关键词提取等
//!
//! # 示例
//!
//! ```no_run
//! use scanners::scan::GameScanner;
//!
//! #[tokio::main]
//! async fn main() {
//!     let game_infos = GameScanner::new()
//!         .with_dlsite_provider().await
//!         .with_igdb_provider("client_id".to_string(), "client_secret".to_string()).await
//!         .scan("D:/Games".to_string()).await;
//!
//!     println!("找到 {} 个游戏", game_infos.len());
//! }
//! ```

// 子模块
mod patterns;
mod utils;
mod game_grouping;
mod scanner;

// 公共导出
pub use scanner::{GameScanner, walk_path};
pub use game_grouping::{PathGroupResult, DirEntryFilter, paths_group};
pub use utils::{extract_version, extract_search_key, find_common_parent_dir, calculate_directory_size_async};