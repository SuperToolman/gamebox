use async_trait::async_trait;
use crate::models::game_meta_data::GameMetadata;
use crate::providers::GameDatabaseProvider;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// IGDB OAuth 令牌响应
#[derive(Debug, Deserialize)]
struct TwitchTokenResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

/// IGDB 封面响应
#[derive(Debug, Deserialize)]
struct IGDBCover {
    image_id: Option<String>,
}

/// IGDB 公司响应
#[derive(Debug, Deserialize)]
struct IGDBInvolvedCompany {
    company: Option<IGDBCompany>,
    developer: Option<bool>,
    publisher: Option<bool>,
}

/// IGDB 公司信息
#[derive(Debug, Deserialize)]
struct IGDBCompany {
    name: Option<String>,
}

/// IGDB 游戏响应
#[derive(Debug, Deserialize)]
struct IGDBGame {
    id: Option<u64>,
    name: Option<String>,
    summary: Option<String>,
    #[serde(rename = "first_release_date")]
    first_release_date: Option<u64>,
    cover: Option<IGDBCover>,
    involved_companies: Option<Vec<IGDBInvolvedCompany>>,
}

/// IGDB 数据库提供者
pub struct IGDBProvider {
    client_id: String,
    client_secret: String,
    access_token: Arc<RwLock<Option<String>>>,
    http_client: reqwest::Client,
}

impl IGDBProvider {
    /// 创建新的 IGDB 提供者（需要客户端ID和密钥）
    pub fn new() -> Self {
        IGDBProvider {
            client_id: String::new(),
            client_secret: String::new(),
            access_token: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::new(),
        }
    }

    /// 使用凭证创建 IGDB 提供者
    pub fn with_credentials(client_id: String, client_secret: String) -> Self {
        IGDBProvider {
            client_id,
            client_secret,
            access_token: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::new(),
        }
    }

    /// 设置凭证
    pub fn set_credentials(&mut self, client_id: String, client_secret: String) {
        self.client_id = client_id;
        self.client_secret = client_secret;
    }

    /// 获取访问令牌
    async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否已有令牌
        {
            let token = self.access_token.read().await;
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
            }
        }

        // 请求新令牌
        let url = format!(
            "https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials",
            self.client_id, self.client_secret
        );

        let response = self.http_client
            .post(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get access token: {}", response.status()).into());
        }

        let token_response: TwitchTokenResponse = response.json().await?;

        // 保存令牌
        {
            let mut token = self.access_token.write().await;
            *token = Some(token_response.access_token.clone());
        }

        Ok(token_response.access_token)
    }
}

impl Default for IGDBProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameDatabaseProvider for IGDBProvider {
    fn name(&self) -> &str {
        "IGDB"
    }

    async fn search(&self, title: &str) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查凭证
        if self.client_id.is_empty() || self.client_secret.is_empty() {
            return Err("IGDB credentials not configured".into());
        }

        // 获取访问令牌
        let access_token = self.get_access_token().await?;

        // 构建 IGDB API 查询（扩展 cover 和 involved_companies 字段）
        let query = format!(
            "search \"{}\"; fields name,summary,first_release_date,cover.image_id,involved_companies.company.name,involved_companies.developer,involved_companies.publisher; limit 10;",
            title.replace('"', "\\\"")
        );

        // 发送请求到 IGDB API
        let response = self.http_client
            .post("https://api.igdb.com/v4/games")
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", access_token))
            .body(query)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("IGDB API error: {}", response.status()).into());
        }

        let games: Vec<IGDBGame> = response.json().await?;

        // 转换为 GameMetadata
        let results: Vec<GameMetadata> = games
            .into_iter()
            .map(|game| {
                let release_date = game.first_release_date.map(|timestamp| {
                    // 转换 Unix 时间戳为年份
                    let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0);
                    datetime.map(|dt| dt.format("%Y").to_string()).unwrap_or_default()
                });

                // 提取开发商和发行商
                let mut developer = None;
                let mut publisher = None;

                if let Some(companies) = &game.involved_companies {
                    for involved in companies {
                        if let Some(company) = &involved.company {
                            if involved.developer.unwrap_or(false) && developer.is_none() {
                                developer = company.name.clone();
                            }
                            if involved.publisher.unwrap_or(false) && publisher.is_none() {
                                publisher = company.name.clone();
                            }
                        }
                    }
                }

                // 构建封面 URL
                let cover_url = game.cover.and_then(|cover| {
                    cover.image_id.map(|image_id| {
                        format!("https://images.igdb.com/igdb/image/upload/t_cover_big/{}.jpg", image_id)
                    })
                });

                GameMetadata {
                    title: game.name,
                    release_date,
                    developer,
                    publisher,
                    description: game.summary,
                    cover_url,
                    genres: None,
                    tags: None,
                }
            })
            .collect();

        Ok(results)
    }

    async fn get_by_id(&self, id: &str) -> Result<GameMetadata, Box<dyn std::error::Error + Send + Sync>> {
        // 检查凭证
        if self.client_id.is_empty() || self.client_secret.is_empty() {
            return Err("IGDB credentials not configured".into());
        }

        // 获取访问令牌
        let access_token = self.get_access_token().await?;

        // 构建查询（扩展字段）
        let query = format!(
            "fields name,summary,first_release_date,cover.image_id,involved_companies.company.name,involved_companies.developer,involved_companies.publisher; where id = {};",
            id
        );

        // 发送请求
        let response = self.http_client
            .post("https://api.igdb.com/v4/games")
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", access_token))
            .body(query)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("IGDB API error: {}", response.status()).into());
        }

        let games: Vec<IGDBGame> = response.json().await?;

        if games.is_empty() {
            return Err(format!("Game with ID {} not found", id).into());
        }

        let game = &games[0];
        let release_date = game.first_release_date.map(|timestamp| {
            let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0);
            datetime.map(|dt| dt.format("%Y").to_string()).unwrap_or_default()
        });

        // 提取开发商和发行商
        let mut developer = None;
        let mut publisher = None;

        if let Some(companies) = &game.involved_companies {
            for involved in companies {
                if let Some(company) = &involved.company {
                    if involved.developer.unwrap_or(false) && developer.is_none() {
                        developer = company.name.clone();
                    }
                    if involved.publisher.unwrap_or(false) && publisher.is_none() {
                        publisher = company.name.clone();
                    }
                }
            }
        }

        // 构建封面 URL
        let cover_url = game.cover.as_ref().and_then(|cover| {
            cover.image_id.as_ref().map(|image_id| {
                format!("https://images.igdb.com/igdb/image/upload/t_cover_big/{}.jpg", image_id)
            })
        });

        Ok(GameMetadata {
            title: game.name.clone(),
            release_date,
            developer,
            publisher,
            description: game.summary.clone(),
            cover_url,
            genres: None,
            tags: None,
        })
    }

    fn priority(&self) -> u32 {
        80  // 欧美游戏优先级较高
    }

    fn supports_game_type(&self, game_type: &str) -> bool {
        matches!(game_type, "western_game" | "aaa_game" | "indie_game" | "all")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_igdb_provider_no_credentials() {
        let provider = IGDBProvider::new();
        let result = provider.search("test game").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "IGDB credentials not configured");
    }

    #[tokio::test]
    async fn test_igdb_provider_priority() {
        let provider = IGDBProvider::new();
        assert_eq!(provider.priority(), 80);
    }

    #[tokio::test]
    async fn test_igdb_provider_supports_game_type() {
        let provider = IGDBProvider::new();
        assert!(provider.supports_game_type("western_game"));
        assert!(provider.supports_game_type("aaa_game"));
        assert!(provider.supports_game_type("all"));
        assert!(!provider.supports_game_type("visual_novel"));
    }
}

