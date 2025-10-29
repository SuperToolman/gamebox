use gamebox::logger::{init_logger, get_logger, LogEvent, LogLevel};
use gamebox::scan::GameScanner;
use gamebox::traits::JsonOutput;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    init_logger(true);
    let logger = get_logger();

    logger.section("游戏数据库搜索测试");

    // 测试搜索功能 - 使用 DLsite 的作品 ID
    let search_key = "Cloud Meadow".to_string();
    
    logger.log(&LogEvent::new(
        LogLevel::Info,
        format!("正在搜索: {}", search_key),
    ));

    let results = GameScanner::new()
        .with_dlsite_provider().await
        .with_igdb_provider(
            "8v3vdro2ps2sw47wp3lu7cjerrqktr".to_string(),
            "4ow0nznnjbvy3tz7f4bzmb5usp1dhf".to_string(),
        )
        .await
        .search(search_key.clone())
        .await?;

    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("找到 {} 条结果", results.len()),
    ));

    // 显示结果
    for (idx, result) in results.iter().enumerate() {
        logger.subsection(&format!("结果 #{}", idx + 1));
        
        println!("  来源: {}", result.source);
        println!("  置信度: {:.2}", result.confidence);
        
        if let Some(title) = &result.info.title {
            println!("  标题: {}", title);
        }
        
        if let Some(developer) = &result.info.developer {
            println!("  开发商: {}", developer);
        }
        
        if let Some(publisher) = &result.info.publisher {
            println!("  发行商: {}", publisher);
        }
        
        if let Some(release_date) = &result.info.release_date {
            println!("  发布日期: {}", release_date);
        }
        
        if let Some(description) = &result.info.description {
            let short_desc = if description.len() > 100 {
                format!("{}...", &description[..100])
            } else {
                description.clone()
            };
            println!("  简介: {}", short_desc);
        }
        
        if let Some(genres) = &result.info.genres {
            println!("  类型: {}", genres.join(", "));
        }
        
        if let Some(cover_url) = &result.info.cover_url {
            println!("  封面: {}", cover_url);
        }
        
        println!();
    }

    // 测试 out_json 功能
    logger.section("测试 JSON 输出功能");

    // 使用默认路径
    logger.log(&LogEvent::new(
        LogLevel::Info,
        "使用默认路径输出 JSON...".to_string(),
    ));

    let default_path = results.out_json::<&str>(None)?;
    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("已保存到: {}", default_path),
    ));

    // 使用自定义路径
    logger.log(&LogEvent::new(
        LogLevel::Info,
        "使用自定义路径输出 JSON...".to_string(),
    ));

    // 重新搜索以获取新的结果（因为之前的 results 已经被 out_json 消费了）
    let results2 = GameScanner::new()
        .with_dlsite_provider().await
        .with_igdb_provider(
            "8v3vdro2ps2sw47wp3lu7cjerrqktr".to_string(),
            "4ow0nznnjbvy3tz7f4bzmb5usp1dhf".to_string(),
        )
        .await
        .search(search_key)
        .await?;

    let custom_path = results2.out_json(Some("my_search_results.json"))?;
    logger.log(&LogEvent::new(
        LogLevel::Success,
        format!("已保存到: {}", custom_path),
    ));

    Ok(())
}

