/// GameBox 使用示例
/// 
/// 本示例展示了 GameScanner 的两种主要用法：
/// 1. scan() - 扫描本地游戏目录并获取元数据
/// 2. search() - 直接搜索游戏数据库

use gamebox::logger::{init_logger, get_logger, LogEvent, LogLevel};
use gamebox::scan::GameScanner;
use gamebox::traits::JsonOutput;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化日志系统
    init_logger(true);
    let logger = get_logger();

    logger.section("GameBox 使用示例");

    // ========================================
    // 示例 1: 直接搜索游戏数据库
    // ========================================
    logger.section("示例 1: 搜索游戏数据库");
    
    let search_results = GameScanner::new()
        .with_dlsite_provider().await
        .with_igdb_provider(
            "your_client_id".to_string(),
            "your_client_secret".to_string(),
        )
        .await
        .search("Elden Ring".to_string())
        .await?;

    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("找到 {} 条搜索结果", search_results.len()),
    ));

    for result in search_results.iter().take(3) {
        if let Some(title) = &result.info.title {
            println!("  - {} (来源: {}, 置信度: {:.2})", 
                title, result.source, result.confidence);
        }
    }

    // ========================================
    // 示例 2: 扫描本地游戏目录
    // ========================================
    logger.section("示例 2: 扫描本地游戏目录");
    
    // 注意：这里使用了一个新的 GameScanner 实例
    // 因为 scan() 和 search() 都会消费 self
    let game_infos = GameScanner::new()
        .with_dlsite_provider().await
        .with_igdb_provider(
            "your_client_id".to_string(),
            "your_client_secret".to_string(),
        )
        .await
        .scan("D:/Games".to_string())
        .await;

    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("扫描完成，找到 {} 个游戏", game_infos.len()),
    ));

    for game in game_infos.iter().take(5) {
        println!("  - {} ({})", game.title, game.dir_path.display());
    }

    // ========================================
    // 示例 3: 只使用特定的数据库提供者
    // ========================================
    logger.section("示例 3: 只使用 DLsite 提供者");
    
    let dlsite_results = GameScanner::new()
        .with_dlsite_provider().await
        .search("RJ01014447".to_string())
        .await?;

    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("DLsite 搜索结果: {} 条", dlsite_results.len()),
    ));

    // ========================================
    // 示例 4: 使用 out_json() 导出结果
    // ========================================
    logger.section("示例 4: 导出 JSON 结果");

    // 搜索并导出到默认路径 (search_result.json)
    let search_results = GameScanner::new()
        .with_dlsite_provider().await
        .search("game name".to_string())
        .await?;

    let path = search_results.out_json::<&str>(None)?;
    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("搜索结果已保存到: {}", path),
    ));

    // 扫描并导出到自定义路径
    let scan_results = GameScanner::new()
        .with_dlsite_provider().await
        .scan("D:/Games".to_string())
        .await;

    let custom_path = scan_results.out_json(Some("my_games.json"))?;
    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("扫描结果已保存到: {}", custom_path),
    ));

    Ok(())
}

