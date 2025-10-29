//! JSON 输出 trait
//!
//! 为扫描和搜索结果提供 JSON 输出功能

use std::fs::File;
use std::io::Write;
use std::path::Path;
use serde::Serialize;

/// JSON 输出 trait
///
/// 为结果类型提供输出为 JSON 文件的功能
pub trait JsonOutput: Serialize {
    /// 获取默认输出文件名
    fn default_filename() -> &'static str;

    /// 输出为 JSON 文件
    ///
    /// # 参数
    /// - `path`: 可选的输出路径，如果为 None 则使用默认路径
    ///
    /// # 返回
    /// - `Ok(String)`: 成功时返回实际使用的文件路径
    /// - `Err`: 失败时返回错误信息
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use gamebox::scan::GameScanner;
    /// use gamebox::traits::JsonOutput;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 使用默认路径
    ///     let game_infos = GameScanner::new()
    ///         .with_dlsite_provider().await
    ///         .scan("D:/Games".to_string()).await;
    ///     
    ///     game_infos.out_json::<&str>(None)?;  // 输出到 ./scan_result.json
    ///
    ///     // 使用自定义路径
    ///     let results = GameScanner::new()
    ///         .with_dlsite_provider().await
    ///         .search("game name".to_string()).await?;
    ///
    ///     results.out_json(Some("my_results.json"))?;  // 输出到 ./my_results.json
    ///     
    ///     Ok(())
    /// }
    /// ```
    fn out_json<P: AsRef<Path>>(&self, path: Option<P>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 确定输出路径
        let output_path = if let Some(p) = path {
            p.as_ref().to_path_buf()
        } else {
            std::path::PathBuf::from(Self::default_filename())
        };

        // 序列化为 JSON
        let json_output = serde_json::to_string_pretty(self)?;

        // 写入文件
        let mut file = File::create(&output_path)?;
        file.write_all(json_output.as_bytes())?;

        // 返回实际使用的路径
        Ok(output_path.display().to_string())
    }
}

// 为 Vec<GameInfo> 实现 JsonOutput
impl JsonOutput for Vec<crate::models::game_info::GameInfo> {
    fn default_filename() -> &'static str {
        "scan_result.json"
    }
}

// 为 Vec<GameQueryResult> 实现 JsonOutput
impl JsonOutput for Vec<crate::providers::GameQueryResult> {
    fn default_filename() -> &'static str {
        "search_result.json"
    }
}

