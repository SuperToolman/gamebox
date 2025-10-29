use gamebox::logger::{LogEvent, LogLevel, get_logger, init_logger};
use gamebox::scan::GameScanner;
use gamebox::traits::JsonOutput;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    init_logger(true);
    let logger = get_logger();

    // 定义扫描路径
    let scan_path = String::from(r"D:\Test\save1\Game");

    logger.section("游戏目录扫描测试");

    // 使用新的链式 API
    let game_infos = GameScanner::new()
        .with_dlsite_provider()
        .await
        .with_igdb_provider(
            "8v3vdro2ps2sw47wp3lu7cjerrqktr".to_string(),
            "4ow0nznnjbvy3tz7f4bzmb5usp1dhf".to_string(),
        )
        .await
        .scan(scan_path)
        .await;

    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("扫描完成，找到 {} 个游戏", game_infos.len()),
    ));

    // 测试 out_json 功能
    logger.section("测试 JSON 输出功能");

    // 使用默认路径
    logger.log(&LogEvent::new(
        LogLevel::Info,
        "使用默认路径输出 JSON...".to_string(),
    ));

    let default_path = game_infos.out_json::<&str>(None)?;
    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("已保存到: {}", default_path),
    ));

    // 使用自定义路径 - 重新扫描
    logger.log(&LogEvent::new(
        LogLevel::Info,
        "使用自定义路径输出 JSON...".to_string(),
    ));

    let game_infos2 = GameScanner::new()
        .with_dlsite_provider()
        .await
        .scan("D:/Games".to_string())
        .await;

    let custom_path = game_infos2.out_json(Some("my_scan_results.json"))?;
    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("已保存到: {}", custom_path),
    ));

    Ok(())
}
