//! 游戏扫描器核心实现
//!
//! 该模块提供了 `GameScanner` 结构体，用于扫描本地游戏文件并通过游戏数据库提供者获取元数据。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::Utc;
use ignore::{DirEntry, Walk, WalkBuilder};

use crate::logger::{get_logger, LogEvent, LogLevel, ScanProgress};
use crate::models::game_info::GameInfo;
use crate::providers::GameDatabaseMiddleware;
use crate::scan::game_grouping::{paths_group, PathGroupResult};
use crate::scan::utils::{calculate_directory_size_async, find_icon_path};

/// 游戏扫描器
///
/// 用于扫描本地游戏文件并通过游戏数据库提供者获取元数据。
///
/// # 示例
///
/// ```no_run
/// use scanners::scan::GameScanner;
///
/// #[tokio::main]
/// async fn main() {
///     let game_infos = GameScanner::new()
///         .with_dlsite_provider().await
///         .with_igdb_provider("client_id".to_string(), "client_secret".to_string()).await
///         .scan("D:/Games".to_string()).await;
///
///     println!("找到 {} 个游戏", game_infos.len());
/// }
/// ```
pub struct GameScanner {
    /// 游戏数据库中间件
    middleware: GameDatabaseMiddleware,
    enable_exe_icon_extract: bool,
}

impl GameScanner {
    /// 创建新的游戏扫描器
    ///
    /// # 返回
    /// 新的 `GameScanner` 实例
    pub fn new() -> Self {
        GameScanner {
            middleware: GameDatabaseMiddleware::new(),
            enable_exe_icon_extract: false,
        }
    }

    /// 注册 DLsite 提供者（链式调用）
    ///
    /// # 返回
    /// 返回 `self` 以支持链式调用
    pub async fn with_dlsite_provider(self) -> Self {
        use crate::providers::dlsite_provider::DLsiteProvider;
        self.middleware
            .register_provider(Arc::new(DLsiteProvider::new()))
            .await;
        self
    }

    /// 注册 IGDB 提供者（链式调用）
    ///
    /// # 参数
    /// - `client_id`: IGDB API 客户端 ID
    /// - `client_secret`: IGDB API 客户端密钥
    ///
    /// # 返回
    /// 返回 `self` 以支持链式调用
    pub async fn with_igdb_provider(self, client_id: String, client_secret: String) -> Self {
        use crate::providers::igdb_provider::IGDBProvider;
        self.middleware
            .register_provider(Arc::new(IGDBProvider::with_credentials(
                client_id,
                client_secret,
            )))
            .await;
        self
    }

    /// 注册 TheGamesDB 提供者（链式调用）
    ///
    /// # 返回
    /// 返回 `self` 以支持链式调用
    pub async fn with_thegamesdb_provider(self) -> Self {
        use crate::providers::thegamesdb_provider::TheGamesDBProvider;
        self.middleware
            .register_provider(Arc::new(TheGamesDBProvider::new()))
            .await;
        self
    }

    pub fn with_win_exe_icon(self) -> Self {
        GameScanner {
            middleware: self.middleware,
            enable_exe_icon_extract: true,
        }
    }

    /// 注册自定义提供者（链式调用）
    ///
    /// # 参数
    /// - `provider`: 实现了 `GameDatabaseProvider` trait 的提供者
    ///
    /// # 返回
    /// 返回 `self` 以支持链式调用
    pub async fn with_provider(
        self,
        provider: Arc<dyn crate::providers::GameDatabaseProvider>,
    ) -> Self {
        self.middleware.register_provider(provider).await;
        self
    }

    /// 执行扫描
    ///
    /// # 参数
    /// - `scan_path`: 要扫描的目录路径
    ///
    /// # 返回
    /// 扫描到的游戏信息列表
    pub async fn scan(self, scan_path: String) -> Vec<GameInfo> {
        self.scan_internal(scan_path).await
    }

    /// 直接搜索游戏数据库
    ///
    /// 此方法不扫描本地文件，而是直接向已注册的数据库提供者查询游戏信息。
    ///
    /// # 参数
    /// - `search_key`: 搜索关键词（游戏名称）
    ///
    /// # 返回
    /// 查询结果列表，按置信度从高到低排序
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use scanners::scan::GameScanner;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let results = GameScanner::new()
    ///         .with_dlsite_provider().await
    ///         .search("Elden Ring".to_string()).await?;
    ///
    ///     for result in results {
    ///         println!("找到: {} (来源: {}, 置信度: {:.2})",
    ///             result.info.title.unwrap_or_default(),
    ///             result.source,
    ///             result.confidence
    ///         );
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn search(
        self,
        search_key: String,
    ) -> Result<Vec<crate::providers::GameQueryResult>, Box<dyn std::error::Error + Send + Sync>> {
        self.middleware
            .search_with_timeout(&search_key, std::time::Duration::from_secs(30))
            .await
    }

    /// 内部扫描实现
    async fn scan_internal(&self, scan_path: String) -> Vec<GameInfo> {
        let mut game_infos: Vec<GameInfo> = Vec::new();

        let logger = get_logger();
        logger.log(&LogEvent::new(
            LogLevel::Info,
            "开始并行扫描 .exe 文件...",
        ));

        // 使用并行遍历收集 .exe 文件路径
        let exe_paths = Arc::new(Mutex::new(Vec::new()));

        {
            let exe_paths_clone = Arc::clone(&exe_paths);
            WalkBuilder::new(&scan_path)
                .threads(num_cpus::get()) // 使用所有 CPU 核心
                .build_parallel()
                .run(|| {
                    let exe_paths = Arc::clone(&exe_paths_clone);
                    Box::new(move |result| {
                        if let Ok(entry) = result {
                            // 只处理文件
                            if let Some(file_type) = entry.file_type() {
                                if file_type.is_file() {
                                    // 只处理 .exe 文件
                                    if let Some(ext) = entry.path().extension() {
                                        if ext == "exe" {
                                            // 存储路径而不是 DirEntry（避免生命周期问题）
                                            if let Ok(mut paths) = exe_paths.lock() {
                                                paths.push(entry.path().to_path_buf());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ignore::WalkState::Continue
                    })
                });
        } // exe_paths_clone 在这里被 drop

        // 提取路径（现在只有一个 Arc 引用）
        let exe_paths = Arc::try_unwrap(exe_paths)
            .expect("Failed to unwrap Arc")
            .into_inner()
            .expect("Failed to unwrap Mutex");

        logger.log(&LogEvent::new(
            LogLevel::Success,
            format!("扫描完成，找到 {} 个 .exe 文件", exe_paths.len()),
        ));

        // 将路径转换回 DirEntry 格式（通过重新遍历）
        let mut exe_dirs: Vec<DirEntry> = Vec::new();
        for path in exe_paths {
            // 使用 Walk 获取 DirEntry
            for result in Walk::new(&path) {
                if let Ok(entry) = result {
                    if entry.path() == path {
                        exe_dirs.push(entry);
                        break;
                    }
                }
            }
        }

        // 对扫描结果分组
        let groups: Vec<PathGroupResult> = paths_group(exe_dirs);

        let logger = get_logger();

        for (idx, item) in groups.iter().enumerate() {
            // 显示进度
            let progress = ScanProgress::new(idx + 1, groups.len(), &item.child_root_name);
            logger.section(&format!("{} - {}", progress.format(), item.child_root_name));

            if item.search_key != item.child_root_name {
                logger.log(&LogEvent::new(
                    LogLevel::Debug,
                    format!("搜索关键词: {}", item.search_key),
                ));
            }

            let start_time = Instant::now();
            match self.middleware.search(&item.search_key).await {
                Ok(game_query_results) => {
                    let duration_ms = start_time.elapsed().as_millis() as u64;

                    // game_query_results包含查询多个游戏数据库所获得的结果，各个来源都不同，数据也不同
                    if game_query_results.is_empty() {
                        logger.log(&LogEvent::new(LogLevel::Warning, "未找到任何结果"));
                    } else {
                        // 处理查询结果
                        self.process_query_results(&game_query_results, duration_ms);
                    }

                    // 构建 GameInfo
                    let game_info = self.build_game_info(item, game_query_results).await;
                    game_infos.push(game_info);
                }
                Err(e) => {
                    logger.log(
                        &LogEvent::new(
                            LogLevel::Error,
                            format!("查询失败: {}", item.child_root_name),
                        )
                        .with_details(e.to_string()),
                    );

                    // 即使查询失败，也创建基本的 GameInfo
                    let game_info = self.build_fallback_game_info(item).await;
                    game_infos.push(game_info);
                }
            }
        }

        logger.section(&format!("扫描完成！共找到 {} 个游戏", game_infos.len()));
        logger.log(&LogEvent::new(
            LogLevel::Success,
            format!("成功扫描 {} 个游戏目录", game_infos.len()),
        ));

        game_infos
    }

    /// 处理查询结果并显示日志
    fn process_query_results(
        &self,
        game_query_results: &[crate::providers::GameQueryResult],
        duration_ms: u64,
    ) {
        let logger = get_logger();

        // 按来源分组结果
        let mut provider_results: std::collections::HashMap<
            String,
            Vec<&crate::providers::GameQueryResult>,
        > = std::collections::HashMap::new();
        for result in game_query_results {
            provider_results
                .entry(result.source.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        // 显示查询摘要
        logger.log(&LogEvent::new(
            LogLevel::Success,
            format!(
                "找到 {} 条结果 (耗时: {}ms)",
                game_query_results.len(),
                duration_ms
            ),
        ));

        // 按提供者显示结果
        for (provider_name, results) in provider_results.iter() {
            logger.subsection(&format!(
                "📦 {} - {} 条结果",
                provider_name,
                results.len()
            ));

            for (idx, result) in results.iter().enumerate() {
                println!(
                    "   [{}/{}] 置信度: {:.2}",
                    idx + 1,
                    results.len(),
                    result.confidence
                );

                if let Some(title) = &result.info.title {
                    println!("       标题: {}", title);
                }
                if let Some(developer) = &result.info.developer {
                    println!("       开发商: {}", developer);
                }
                if let Some(publisher) = &result.info.publisher {
                    println!("       发行商: {}", publisher);
                }
                if let Some(release_date) = &result.info.release_date {
                    println!("       发布日期: {}", release_date);
                }
                if let Some(genres) = &result.info.genres {
                    println!("       类型: {}", genres.join(", "));
                }
                if let Some(cover_url) = &result.info.cover_url {
                    println!("       封面: {}", cover_url);
                }
                println!();
            }
        }
    }


    /// 从查询结果构建 GameInfo
    async fn build_game_info(
        &self,
        item: &PathGroupResult,
        game_query_results: Vec<crate::providers::GameQueryResult>,
    ) -> GameInfo {
        // 合并所有数据库的结果
        let mut title = None; // 优先使用置信度最高的结果的标题
        let mut cover_urls = Vec::new();
        let mut description = None;
        let mut release_date = None;
        let mut developer = None;
        let mut publisher = None;
        let mut tabs = None;
        let platform = None;

        // 从所有查询结果中收集数据（优先使用置信度最高的）
        for result in game_query_results.iter() {
            // 如果还没有标题，使用第一个（置信度最高的）结果的标题
            if title.is_none() && result.info.title.is_some() {
                title = result.info.title.clone();
            }
            // 收集所有封面URL
            if let Some(cover_url) = &result.info.cover_url {
                if !cover_urls.contains(cover_url) {
                    cover_urls.push(cover_url.clone());
                }
            }

            // 如果还没有描述，使用第一个有描述的结果
            if description.is_none() && result.info.description.is_some() {
                description = result.info.description.clone();
            }

            // 如果还没有发布日期，使用第一个有发布日期的结果
            if release_date.is_none() && result.info.release_date.is_some() {
                release_date = result.info.release_date.clone();
            }

            // 如果还没有开发商，使用第一个有开发商的结果
            if developer.is_none() && result.info.developer.is_some() {
                developer = result.info.developer.clone();
            }

            // 如果还没有发行商，使用第一个有发行商的结果
            if publisher.is_none() && result.info.publisher.is_some() {
                publisher = result.info.publisher.clone();
            }

            // 收集所有标签
            if let Some(genres) = &result.info.genres {
                let genres_str = genres.join(", ");
                if tabs.is_none() {
                    tabs = Some(genres_str);
                } else if let Some(existing_tabs) = &tabs {
                    // 合并标签，避免重复
                    let mut all_tabs: Vec<String> = existing_tabs
                        .split(", ")
                        .map(|s| s.to_string())
                        .collect();
                    for genre in genres {
                        if !all_tabs.contains(genre) {
                            all_tabs.push(genre.clone());
                        }
                    }
                    tabs = Some(all_tabs.join(", "));
                }
            }

            // 收集所有标签（从tags字段）
            if let Some(tags) = &result.info.tags {
                let tags_str = tags.join(", ");
                if tabs.is_none() {
                    tabs = Some(tags_str);
                } else if let Some(existing_tabs) = &tabs {
                    // 合并标签，避免重复
                    let mut all_tabs: Vec<String> = existing_tabs
                        .split(", ")
                        .map(|s| s.to_string())
                        .collect();
                    for tag in tags {
                        if !all_tabs.contains(tag) {
                            all_tabs.push(tag.clone());
                        }
                    }
                    tabs = Some(all_tabs.join(", "));
                }
            }
        }

        // 游戏目录路径（root_path 已经是完整的游戏根目录路径）
        let dir_path = PathBuf::from(&item.root_path);

        let byte_size = calculate_directory_size_async(dir_path.clone()).await;
        let start_path_defualt = {
            let mut exe_paths: Vec<(usize, String)> = item
                .child_path
                .iter()
                .filter(|p| p.to_lowercase().ends_with(".exe"))
                .map(|p| {
                    let depth = p.matches('/').count() + p.matches('\\').count();
                    (depth, p.clone())
                })
                .collect();
            exe_paths.sort_by(|a, b| a.0.cmp(&b.0));
            exe_paths
                .first()
                .map(|(_, p)| p.clone())
                .or_else(|| item.child_path.first().cloned())
                .unwrap_or_default()
        };
        let icon_path = {
            let p = start_path_defualt.clone();
            find_icon_path(
                dir_path.clone(),
                &item.child_path,
                self.enable_exe_icon_extract,
                &item.child_root_name,
                Some(&p),
            )
            .await
            .unwrap_or_default()
        };

        let parsed_release_date = if let Some(date_str) = release_date {
            // 尝试解析日期字符串
            chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                .or_else(|| {
                    // 尝试只解析年份
                    date_str.parse::<i32>().ok().and_then(|year| {
                        chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                            .and_then(|d| d.and_hms_opt(0, 0, 0))
                            .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                    })
                })
                .unwrap_or_else(Utc::now)
        } else {
            Utc::now()
        };

        let final_title = title.unwrap_or_else(|| item.child_root_name.clone());

        // sort start_path shallow → deep, tie by length
        let mut sorted_child_path: Vec<String> = item.child_path.clone();
        sorted_child_path.sort_by(|a, b| {
            let da = a.matches('/').count() + a.matches('\\').count();
            let db = b.matches('/').count() + b.matches('\\').count();
            da.cmp(&db).then(a.len().cmp(&b.len()))
        });
        GameInfo {
            title: final_title,
            sub_title: item.child_root_name.clone(), // 副标题始终使用本地目录名
            version: item.version.clone(),
            cover_urls,
            dir_path,
            start_path: sorted_child_path,
            start_path_defualt,
            description,
            release_date: parsed_release_date,
            developer,
            publisher,
            tabs,
            platform,
            byte_size,
            scan_time: Utc::now(),
            icon_path,
        }
    }

    /// 构建回退的 GameInfo（当查询失败时）
    async fn build_fallback_game_info(&self, item: &PathGroupResult) -> GameInfo {
        let dir_path = PathBuf::from(&item.root_path);
        let byte_size = calculate_directory_size_async(dir_path.clone()).await;
        let start_path_defualt = {
            let mut exe_paths: Vec<(usize, String)> = item
                .child_path
                .iter()
                .filter(|p| p.to_lowercase().ends_with(".exe"))
                .map(|p| {
                    let depth = p.matches('/').count() + p.matches('\\').count();
                    (depth, p.clone())
                })
                .collect();
            exe_paths.sort_by(|a, b| a.0.cmp(&b.0));
            exe_paths
                .first()
                .map(|(_, p)| p.clone())
                .or_else(|| item.child_path.first().cloned())
                .unwrap_or_default()
        };
        let icon_path = {
            let p = start_path_defualt.clone();
            find_icon_path(
                dir_path.clone(),
                &item.child_path,
                self.enable_exe_icon_extract,
                &item.child_root_name,
                Some(&p),
            )
            .await
            .unwrap_or_default()
        };

        // sort start_path shallow → deep, tie by length
        let mut sorted_child_path: Vec<String> = item.child_path.clone();
        sorted_child_path.sort_by(|a, b| {
            let da = a.matches('/').count() + a.matches('\\').count();
            let db = b.matches('/').count() + b.matches('\\').count();
            da.cmp(&db).then(a.len().cmp(&b.len()))
        });
        GameInfo {
            title: item.child_root_name.clone(),
            sub_title: item.child_root_name.clone(), // 副标题始终使用本地目录名
            version: item.version.clone(),
            cover_urls: Vec::new(),
            dir_path,
            start_path: sorted_child_path,
            start_path_defualt,
            description: None,
            release_date: Utc::now(),
            developer: None,
            publisher: None,
            tabs: None,
            platform: None,
            byte_size,
            scan_time: Utc::now(),
            icon_path,
        }
    }
}

/// 保留旧的 walk_path 函数以保持向后兼容
#[deprecated(since = "0.2.0", note = "请使用 GameScanner::new().scan(path) 代替")]
pub async fn walk_path(root_path: String) -> Vec<GameInfo> {
    GameScanner::new().scan(root_path).await
}
