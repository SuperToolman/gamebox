pub mod dlsite_provider;
pub mod igdb_provider;
pub mod thegamesdb_provider;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use serde::{Serialize, Deserialize};
use crate::models::game_meta_data::GameMetadata;
use crate::logger::{get_logger, LogEvent, LogLevel};

/// 计算两个字符串的相似度（Levenshtein 距离）
fn string_similarity(s1: &str, s2: &str) -> f32 {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }
    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let max_len = len1.max(len2);
    let distance = levenshtein_distance(s1, s2);

    1.0 - (distance as f32 / max_len as f32)
}

/// 计算 Levenshtein 距离（优化版：空间复杂度 O(m) 而非 O(n*m)）
/// 使用滚动数组技术，只保留两行数据
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    // 边界情况
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    // 使用两个一维数组代替二维矩阵（滚动数组技术）
    let mut prev_row = vec![0; len2 + 1];
    let mut curr_row = vec![0; len2 + 1];

    // 初始化第一行
    for j in 0..=len2 {
        prev_row[j] = j;
    }

    // 逐行计算
    for i in 1..=len1 {
        curr_row[0] = i; // 每行的第一列

        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            curr_row[j] = (prev_row[j] + 1)           // 删除
                .min(curr_row[j - 1] + 1)             // 插入
                .min(prev_row[j - 1] + cost);         // 替换
        }

        // 交换行（避免内存分配）
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[len2]
}

/// 计算搜索结果的置信度
/// 基于标题匹配度和数据完整度
fn calculate_confidence(search_title: &str, metadata: &GameMetadata) -> f32 {
    let mut confidence = 0.0;

    // 1. 标题匹配度 (最高 0.7)
    if let Some(title) = &metadata.title {
        let search_lower = search_title.to_lowercase();
        let title_lower = title.to_lowercase();

        // 完全匹配
        if search_lower == title_lower {
            confidence += 0.7;
        }
        // 搜索词是标题的子串（精确包含）
        else if title_lower.contains(&search_lower) {
            // 根据长度比例调整置信度
            let ratio = search_lower.len() as f32 / title_lower.len() as f32;
            confidence += 0.5 + (ratio * 0.2);
        }
        // 标题是搜索词的子串
        else if search_lower.contains(&title_lower) {
            let ratio = title_lower.len() as f32 / search_lower.len() as f32;
            confidence += 0.4 + (ratio * 0.2);
        }
        // 使用字符串相似度算法
        else {
            let similarity = string_similarity(&search_lower, &title_lower);

            // 如果相似度很高，给予较高置信度
            if similarity > 0.8 {
                confidence += 0.5 * similarity;
            } else if similarity > 0.5 {
                confidence += 0.3 * similarity;
            } else {
                // 尝试部分匹配（词语重叠）
                let search_words: Vec<&str> = search_lower.split_whitespace().collect();
                let title_words: Vec<&str> = title_lower.split_whitespace().collect();
                let mut matches = 0;
                let mut total_match_len = 0;

                for sw in &search_words {
                    for tw in &title_words {
                        if tw.contains(sw) || sw.contains(tw) {
                            matches += 1;
                            total_match_len += sw.len().min(tw.len());
                            break;
                        }
                    }
                }

                if !search_words.is_empty() {
                    let match_ratio = matches as f32 / search_words.len() as f32;
                    let length_ratio = total_match_len as f32 / search_lower.len() as f32;
                    confidence += 0.2 * match_ratio + 0.1 * length_ratio;
                }
            }
        }
    }

    // 2. 数据完整度 (最高 0.3)
    let mut completeness = 0.0;
    if metadata.title.is_some() { completeness += 0.08; }
    if metadata.cover_url.is_some() { completeness += 0.05; }
    if metadata.description.is_some() { completeness += 0.04; }
    if metadata.release_date.is_some() { completeness += 0.04; }
    if metadata.developer.is_some() { completeness += 0.04; }
    if metadata.publisher.is_some() { completeness += 0.03; }
    if metadata.genres.is_some() { completeness += 0.01; }
    if metadata.tags.is_some() { completeness += 0.01; }

    confidence += completeness;

    // 确保置信度在 0.0 到 1.0 之间
    confidence.max(0.0).min(1.0)
}

/// 游戏中间件
/// 游戏数据库查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameQueryResult {
    /// 游戏元数据
    pub info: GameMetadata,
    /// 来源（游戏数据库提供者）
    pub source: String,
    /// 置信度
    pub confidence: f32,
}


/// 游戏数据库提供者特征
#[async_trait]
pub trait GameDatabaseProvider: Send + Sync {
    /// 获取提供者名称
    fn name(&self) -> &str;

    /// 搜索游戏
    async fn search(&self, title: &str) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error>>;

    /// 获取游戏详情（如果支持）
    async fn get_by_id(&self, _id: &str) -> Result<GameMetadata, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    /// 获取提供者的优先级（0-100，越高越优先）
    fn priority(&self) -> u32 {
        50
    }

    /// 是否支持该类型的游戏
    fn supports_game_type(&self, _game_type: &str) -> bool {
        true
    }
}




pub struct GameDatabaseMiddleware {
    providers: Arc<RwLock<Vec<Arc<dyn GameDatabaseProvider>>>>,
    cache: Arc<RwLock<HashMap<String, Vec<GameQueryResult>>>>,  // 修改为存储 Vec
    cache_ttl: std::time::Duration,
    /// API 速率限制器：限制并发 API 请求数量
    /// 默认最多同时进行 5 个 API 请求，避免触发速率限制
    rate_limiter: Arc<Semaphore>,
}

impl GameDatabaseMiddleware {
    /// 创建新的游戏数据库中间件（不注册任何提供者）
    pub fn new() -> Self {
        GameDatabaseMiddleware {
            providers: Arc::new(RwLock::new(Vec::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(3600), // 1 小时缓存
            rate_limiter: Arc::new(Semaphore::new(5)), // 最多同时 5 个 API 请求
        }
    }

    /// 注册游戏数据库提供者
    pub async fn register_provider(&self, provider: Arc<dyn GameDatabaseProvider>) {
        let mut providers = self.providers.write().await;
        providers.push(provider);
        // 按优先级排序
        providers.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// 注销数据库提供者
    pub async fn unregister_provider(&self, name: &str) {
        let mut providers = self.providers.write().await;
        providers.retain(|p| p.name() != name);
    }

    /// 搜索游戏
    pub async fn search(&self, title: &str) -> Result<Vec<GameQueryResult>, Box<dyn std::error::Error>> {
        self.search_with_timeout(title, std::time::Duration::from_secs(30)).await
    }

    /// 搜索游戏（带超时）
    pub async fn search_with_timeout(
        &self,
        title: &str,
        timeout: std::time::Duration
    ) -> Result<Vec<GameQueryResult>, Box<dyn std::error::Error>> {
        let logger = get_logger();

        // 检查缓存
        let cache = self.cache.read().await;
        if let Some(cached_results) = cache.get(title) {
            logger.log(&LogEvent::new(
                LogLevel::Info,
                format!("从缓存获取: {} 条结果", cached_results.len())
            ));
            return Ok(cached_results.clone());  // 返回所有缓存的结果
        }
        drop(cache);

        let providers = self.providers.read().await;
        let mut results = Vec::new();

        // 并发查询所有提供者（使用速率限制器）
        let mut futures = Vec::new();
        for provider in providers.iter() {
            let provider = Arc::clone(provider);
            let title_clone = title.to_string();
            let provider_name = provider.name().to_string();
            let rate_limiter = Arc::clone(&self.rate_limiter);

            futures.push(async move {
                // 获取速率限制许可（最多同时 5 个请求）
                let _permit = rate_limiter.acquire().await.unwrap();

                match provider.search(&title_clone).await {
                    Ok(games) => {
                        games.into_iter().map(|info| {
                            // 动态计算置信度
                            let confidence = calculate_confidence(&title_clone, &info);

                            GameQueryResult {
                                info,
                                source: provider_name.clone(),
                                confidence,
                            }
                        }).collect::<Vec<_>>()
                    },
                    Err(_e) => {
                        Vec::new()
                    },
                }
                // _permit 在这里自动释放
            });
        }

        // 等待所有查询完成（带超时）
        let query_future = futures::future::join_all(futures);
        let query_results = match tokio::time::timeout(timeout, query_future).await {
            Ok(results) => results,
            Err(_) => {
                logger.log(&LogEvent::new(
                    LogLevel::Warning,
                    "查询超时"
                ));
                return Err("查询超时".into());
            }
        };

        for query_result in query_results {
            results.extend(query_result);
        }

        // 按置信度排序（从高到低）
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // 缓存所有结果
        if !results.is_empty() {
            let mut cache = self.cache.write().await;
            cache.insert(title.to_string(), results.clone());
        }

        Ok(results)
    }

    /// 通过 ID 获取游戏
    pub async fn get_by_id(&self, id: &str) -> Result<GameQueryResult, Box<dyn std::error::Error>> {
        let providers = self.providers.read().await;

        for provider in providers.iter() {
            match provider.get_by_id(id).await {
                Ok(info) => {
                    return Ok(GameQueryResult {
                        info,
                        source: provider.name().to_string(),
                        confidence: 0.95,
                    });
                },
                Err(_) => continue,
            }
        }

        Err("Game not found".into())
    }

    /// 获取所有提供者
    pub async fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.iter().map(|p| p.name().to_string()).collect()
    }

    /// 清空缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// 获取缓存大小
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}