use async_trait::async_trait;
use crate::models::game_meta_data::GameMetadata;
use crate::providers::GameDatabaseProvider;

/// TheGamesDB 数据库提供者
pub struct TheGamesDBProvider {
    // 可以添加配置
}

impl TheGamesDBProvider {
    pub fn new() -> Self {
        TheGamesDBProvider {}
    }
}

impl Default for TheGamesDBProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameDatabaseProvider for TheGamesDBProvider {
    fn name(&self) -> &str {
        "TheGamesDB"
    }

    async fn search(&self, title: &str) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error>> {
        // TODO: 集成 TheGamesDB API
        // 这里是示例实现
        Ok(vec![GameMetadata {
            title: Some(title.to_string()),
            release_date: Some("2024".to_string()),
            developer: Some("TheGamesDB Developer".to_string()),
            publisher: None,
            description: Some("Game from TheGamesDB".to_string()),
            cover_url: None,
            genres: Some(vec!["Adventure".to_string()]),
            tags: None,
        }])
    }

    async fn get_by_id(&self, id: &str) -> Result<GameMetadata, Box<dyn std::error::Error>> {
        // TODO: 通过 TheGamesDB ID 获取游戏信息
        Ok(GameMetadata {
            title: Some(format!("TheGamesDB Game {}", id)),
            release_date: Some("2024".to_string()),
            developer: Some("TheGamesDB Developer".to_string()),
            publisher: None,
            description: Some("Game from TheGamesDB".to_string()),
            cover_url: None,
            genres: Some(vec!["Adventure".to_string()]),
            tags: None,
        })
    }

    fn priority(&self) -> u32 {
        70  // 经典游戏优先级中等
    }

    fn supports_game_type(&self, game_type: &str) -> bool {
        matches!(game_type, "classic_game" | "retro_game" | "multi_platform" | "all")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thegamesdb_provider_search() {
        let provider = TheGamesDBProvider::new();
        let results = provider.search("test game").await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, Some("test game".to_string()));
    }

    #[tokio::test]
    async fn test_thegamesdb_provider_get_by_id() {
        let provider = TheGamesDBProvider::new();
        let result = provider.get_by_id("12345").await.unwrap();
        assert_eq!(result.title, Some("TheGamesDB Game 12345".to_string()));
    }

    #[tokio::test]
    async fn test_thegamesdb_provider_priority() {
        let provider = TheGamesDBProvider::new();
        assert_eq!(provider.priority(), 70);
    }

    #[tokio::test]
    async fn test_thegamesdb_provider_supports_game_type() {
        let provider = TheGamesDBProvider::new();
        assert!(provider.supports_game_type("classic_game"));
        assert!(provider.supports_game_type("retro_game"));
        assert!(provider.supports_game_type("all"));
        assert!(!provider.supports_game_type("visual_novel"));
    }
}

