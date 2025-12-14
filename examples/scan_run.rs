use gamebox::logger::{init_logger, get_logger, LogEvent, LogLevel};
use gamebox::scan::GameScanner;
use gamebox::traits::JsonOutput;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logger(true);
    let logger = get_logger();
    let scan_path = String::from(r"D:\Test\save1\Game");

    logger.section("指定路径扫描运行（含图标逻辑）");
    logger.log(&LogEvent::new(LogLevel::Info, format!("扫描路径: {}", &scan_path)));

    let game_infos = GameScanner::new()
        .with_dlsite_provider()
        .await
        .with_win_exe_icon()
        .scan(scan_path)
        .await;

    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("扫描完成，共 {} 个游戏", game_infos.len()),
    ));

    for (i, game) in game_infos.iter().enumerate() {
        println!(
            "  [{}] {} | icon: {} | dir: {}",
            i + 1,
            game.title,
            game.icon_path,
            game.dir_path.display()
        );
    }

    let out = game_infos.out_json::<&str>(None)?;
    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("结果已导出: {}", out),
    ));

    Ok(())
}
