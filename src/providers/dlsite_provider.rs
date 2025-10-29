use async_trait::async_trait;
use dlsite_gamebox::DlsiteClient;
use dlsite_gamebox::client::search::SearchProductQuery;
use dlsite_gamebox::interface::query::SexCategory;
use crate::models::game_meta_data::GameMetadata;
use crate::providers::GameDatabaseProvider;

/// DLsite 数据库提供者
pub struct DLsiteProvider {
    // 这里可以添加 DLsite 客户端配置
    dlsite_client: DlsiteClient,
}

impl DLsiteProvider {
    pub fn new() -> Self {
        DLsiteProvider {
            dlsite_client: DlsiteClient::default(),
        }
    }
}

impl Default for DLsiteProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameDatabaseProvider for DLsiteProvider {
    fn name(&self) -> &str {
        "DLsite"
    }

    /// 通过标题查找
    async fn search(&self, title: &str) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error + Send + Sync>> {
        // 使用 dlsite 库的搜索功能（新版 API）
        let search_query = SearchProductQuery {
            sex_category: Some(vec![SexCategory::Male]),
            keyword: Some(title.to_string()),
            ..Default::default()
        };

        match self.dlsite_client.search().search_product(&search_query).await {
            Ok(search_result) => {
                // 只对前3个结果获取详细信息，避免过多API请求
                let mut results = Vec::new();

                for (index, product) in search_result.products.into_iter().enumerate() {
                    // 只获取前3个结果的详细信息
                    if index < 3 {
                        // 尝试获取详细信息（新版 API）
                        match self.dlsite_client.product_api().get(&product.id).await {
                            Ok(detailed_product) => {
                                // 调试输出：查看 API 返回的原始数据
                                eprintln!("\n=== DLsite API 详细信息 ===");
                                eprintln!("Product ID: {}", product.id);
                                eprintln!("work_name: {}", detailed_product.work_name);
                                eprintln!("intro: {:?}", detailed_product.intro);
                                eprintln!("regist_date: {:?}", detailed_product.regist_date);
                                eprintln!("genres count: {}", detailed_product.genres.len());
                                for (i, genre) in detailed_product.genres.iter().enumerate() {
                                    eprintln!("  genre[{}]: {:?}", i, genre);
                                }
                                eprintln!("maker_name: {}", detailed_product.maker_name);
                                eprintln!("creators: {:?}", detailed_product.creators);
                                eprintln!("========================\n");

                                results.push(GameMetadata {
                                    title: Some(detailed_product.work_name),
                                    cover_url: Some(product.thumbnail_url),  // 使用搜索结果的缩略图
                                    description: detailed_product.intro,
                                    release_date: detailed_product.regist_date,
                                    developer: detailed_product.creators.as_ref()
                                        .and_then(|c| c.voice_by.as_ref())
                                        .and_then(|v| v.first())
                                        .map(|v| v.name.clone()),
                                    publisher: Some(detailed_product.maker_name),
                                    genres: if detailed_product.genres.is_empty() {
                                        None
                                    } else {
                                        Some(detailed_product.genres.into_iter().map(|genre| genre.name).collect())
                                    },
                                    tags: None,
                                });
                            }
                            Err(_) => {
                                // 如果获取详细信息失败，使用搜索结果的基本信息
                                results.push(GameMetadata {
                                    title: Some(product.title),
                                    cover_url: Some(product.thumbnail_url),
                                    description: None,
                                    release_date: None,
                                    developer: product.creator,
                                    publisher: Some(product.circle_name),
                                    genres: None,
                                    tags: None,
                                });
                            }
                        }
                    } else {
                        // 对于其他结果，只使用搜索结果的基本信息
                        results.push(GameMetadata {
                            title: Some(product.title),
                            cover_url: Some(product.thumbnail_url),
                            description: None,
                            release_date: None,
                            developer: product.creator,
                            publisher: Some(product.circle_name),
                            genres: None,
                            tags: None,
                        });
                    }
                }

                Ok(results)
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    /// 通过ID查找，在Dlsite中是指它专用的站点作品的ID，如：RJ01014447
    async fn get_by_id(&self, id: &str) -> Result<GameMetadata, Box<dyn std::error::Error + Send + Sync>> {
        // 使用 dlsite 库的 API 获取游戏详细信息（新版 API）
        match self.dlsite_client.product_api().get(id).await {
            Ok(product) => {
                Ok(GameMetadata {
                    title: Some(product.work_name),
                    cover_url: None,
                    description: product.intro,
                    release_date: product.regist_date,
                    developer: product.creators.as_ref().and_then(|c| c.voice_by.as_ref()).and_then(|v| v.first()).map(|v| v.name.clone()),
                    publisher: Some(product.maker_name),
                    genres: if product.genres.is_empty() {
                        None
                    } else {
                        Some(product.genres.into_iter().map(|genre| genre.name).collect())
                    },
                    tags: None,
                })
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    fn priority(&self) -> u32 {
        90  // 日式游戏优先级最高
    }
    
    /// 支持的游戏类型
    fn supports_game_type(&self, game_type: &str) -> bool {
        matches!(game_type, "visual_novel" | "japanese_rpg" | "doujin" | "all")
    }
}