use serde::{Deserialize, Serialize};

/// 游戏元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    /// 游戏标题
    pub title: Option<String>,
    /// 封面URL
    pub cover_url: Option<String>,
    /// 游戏描述
    pub description: Option<String>,
    /// 发布日期
    pub release_date: Option<String>,
    /// 开发商
    pub developer: Option<String>,
    /// 发行商
    pub publisher: Option<String>,
    /// 游戏类型
    pub genres: Option<Vec<String>>,
    /// 游戏标签
    pub tags: Option<Vec<String>>,
}

/// 提供默认值的trait
impl Default for GameMetadata {
    fn default() -> GameMetadata {
        GameMetadata {
            title: None,
            cover_url: None,
            description: None,
            release_date: None,
            developer: None,
            publisher: None,
            genres: None,
            tags: None,
        }
    }
}

