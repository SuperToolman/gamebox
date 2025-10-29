use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// 游戏信息结构体：这个结构体是扫描以后最终呈现的信息项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    /// 游戏标题：优先使用数据库查询结果（置信度最高），如果未找到则使用本地目录名
    pub title: String,
    /// 游戏副标题：本地扫描的目录名，由PathGroupResult.child_root_name提供
    pub sub_title: String,
    /// 游戏版本：从PathGroupResult.child_root_name中提取的版本号
    pub version: Option<String>,
    /// 游戏封面：由GameMetadata提供，从各个平台刮削的图片封面
    pub cover_urls: Vec<String>,
    /// 游戏目录：有本地扫的结果项PathGroupResult.root_path + PathGroupResult.child_root_name提供
    pub dir_path: PathBuf,
    /// 游戏启动项：一般一个游戏目录有多个启动项，扫描结果不知道哪个才是游戏的真正启动文件，因此全部收集，由PathGrouopResult.child_path提供
    pub start_path: Vec<String>,
    /// 游戏默认启动项：由start_path中的第一个项提供
    pub start_path_defualt: String,
    /// 游戏介绍：由GameMetadata提供，从各个平台刮削的游戏介绍
    pub description: Option<String>,
    /// 游戏发行日期：由GameMetadata提供，从各个平台刮削的游戏发行日期
    pub release_date: DateTime<Utc>,
    /// 游戏开发商：由GameMetadata提供，从各个平台刮削的游戏开发商
    pub developer: Option<String>,
    /// 游戏发行商：由GameMetadata提供，从各个平台刮削的游戏发行商
    pub publisher: Option<String>,
    /// 游戏标签：由GameMetadata提供，从各个平台刮削的游戏标签
    pub tabs: Option<String>,
    /// 游戏平台：由GameMetadata提供，从各个平台刮削的游戏平台
    pub platform: Option<String>,
    /// 游戏大小：由本地扫描结果提供，PathGroupResult.child_path中所有文件的大小累加
    pub byte_size: u64,
    /// 扫描时间：由本地扫描结果提供，即当前时间
    pub scan_time: DateTime<Utc>,
}

impl GameInfo {
    pub fn new() -> Self {
        GameInfo {
            title: String::new(),
            sub_title: String::new(),
            version: None,
            cover_urls: Vec::new(),
            dir_path: PathBuf::new(),
            start_path: Vec::new(),
            start_path_defualt: String::new(),
            description: None,
            release_date: Utc::now(),
            developer: None,
            publisher: None,
            tabs: None,
            platform: None,
            byte_size: 0,
            scan_time: Utc::now(),
        }
    }

    /// 开始游戏
    ///
    /// # 参数
    /// * `index` - 可选的启动项索引，如果为 None 则使用默认启动项
    ///
    /// # 返回值
    /// * `Ok((bool, String))` - 成功时返回 (true, 完整路径)
    /// * `Err(String)` - 失败时返回错误信息
    pub fn start_game(&self, index: Option<usize>) -> Result<(bool, String), String> {
        // 检查是否有可用的启动项
        if self.start_path.is_empty() {
            return Err("游戏没有可启动项".to_string());
        }

        // 确定要使用的启动路径
        let start_path = if let Some(idx) = index {
            // 使用指定索引的启动项
            if idx >= self.start_path.len() {
                return Err(format!("索引越界: {} (总共 {} 个启动项)", idx, self.start_path.len()));
            }
            &self.start_path[idx]
        } else if !self.start_path_defualt.is_empty() {
            // 使用配置的默认启动项
            &self.start_path_defualt
        } else {
            // 使用第一个启动项作为默认
            &self.start_path[0]
        };

        // 构建完整路径
        let full_path = self.dir_path.join(start_path);

        // 判断文件是否存在
        if !full_path.exists() {
            return Err(format!("启动项不存在: {}", full_path.display()));
        }

        // 启动游戏进程
        match Command::new(&full_path)
            .current_dir(&self.dir_path)  // 设置工作目录为游戏目录
            .spawn()
        {
            Ok(_child) => {
                // 游戏进程已启动，返回成功和路径
                Ok((true, full_path.display().to_string()))
            }
            Err(e) => {
                Err(format!("启动游戏失败: {} - {}", full_path.display(), e))
            }
        }
    }
}
