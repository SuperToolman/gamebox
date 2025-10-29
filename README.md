# GameBox

[![Crates.io](https://img.shields.io/crates/v/gamebox.svg)](https://crates.io/crates/gamebox)
[![Documentation](https://docs.rs/gamebox/badge.svg)](https://docs.rs/gamebox)
[![License](https://img.shields.io/crates/l/gamebox.svg)](LICENSE)

**GameBox** 是一个功能强大的 Rust 库，专为游戏收藏管理而设计。它能够自动扫描本地游戏目录，智能识别游戏文件，并从多个游戏数据库（DLsite、IGDB、TheGamesDB）获取详细的游戏元数据信息。

## 项目简介

如果你有大量的游戏散落在硬盘的各个角落，想要整理和管理它们，GameBox 就是为你准备的工具。它不仅能帮你找到所有游戏，还能自动获取游戏的封面、简介、发行日期等详细信息，让你的游戏库井井有条。

### 核心能力

- **自动发现游戏**：递归扫描指定目录，自动识别游戏可执行文件（.exe），并智能分组到对应的游戏根目录
- **多源元数据获取**：同时查询多个游戏数据库，获取最全面、最准确的游戏信息
- **智能匹配算法**：使用 Levenshtein 距离算法进行模糊匹配，即使游戏名称有差异也能准确识别
- **高性能设计**：利用多线程并行扫描文件，异步并发查询 API，处理大型游戏库也能快速完成
- **灵活易用**：采用流式 API 设计，链式调用，代码简洁优雅

### 适用场景

- 🎮 **游戏收藏管理**：整理和管理本地游戏库，自动获取游戏信息
- 📚 **游戏数据库构建**：为游戏启动器、游戏管理软件提供数据支持
- 🔍 **游戏信息查询**：快速查询游戏的详细信息、封面图片等
- 📊 **游戏统计分析**：导出游戏数据进行统计分析
- 🛠️ **自定义工具开发**：作为基础库集成到你的游戏相关工具中

## Features

- 🔍 **Smart Game Scanning** - Automatically scans directories for game executables and intelligently groups them by game root directory
- 🌐 **Multiple Database Providers** - Supports DLsite, IGDB, and TheGamesDB with extensible provider system
- 🎯 **Intelligent Matching** - Uses Levenshtein distance algorithm for fuzzy title matching with confidence scoring
- ⚡ **High Performance** - Parallel file scanning using multi-threading and concurrent API queries
- 📦 **Flexible API** - Fluent builder pattern with method chaining for easy configuration
- 💾 **Smart Caching** - Built-in result caching with configurable TTL (1 hour default)
- 🚦 **Rate Limiting** - Automatic API rate limiting to prevent hitting provider limits
- 📊 **JSON Export** - Export scan and search results to JSON format
- 🔧 **Extensible** - Easy to add custom game database providers

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gamebox = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## 核心设计

GameBox 的设计遵循以下核心原则，确保库的易用性、可扩展性和高性能：

### 1. 流式 API 设计（Fluent API）

采用构建器模式（Builder Pattern）和方法链（Method Chaining），让代码更加简洁直观：

```rust
GameScanner::new()                    // 创建扫描器实例
    .with_dlsite_provider().await     // 添加 DLsite 数据源
    .with_igdb_provider(...).await    // 添加 IGDB 数据源
    .scan(path).await                 // 执行扫描
```

**设计优势**：
- 无需多次声明变量，一气呵成
- 配置和执行分离，灵活组合不同的数据源
- 类型安全，编译期检查错误

### 2. 异步优先（Async-First）

基于 Tokio 异步运行时，充分利用异步 I/O 的优势：

- **文件扫描**：使用多线程并行遍历目录树
- **API 查询**：并发请求多个数据源，而不是串行等待
- **资源高效**：异步任务不会阻塞线程，可以同时处理大量请求

**性能对比**：
- 串行查询 3 个数据源：3-6 秒
- 并发查询 3 个数据源：1-2 秒（提升 3-5 倍）

### 3. 中间件模式（Middleware Pattern）

`GameDatabaseMiddleware` 作为中间层，统一管理多个数据源：

- **统一接口**：对外提供一致的 API，隐藏底层复杂性
- **优先级排序**：根据数据源优先级自动选择最佳结果
- **并发控制**：使用信号量（Semaphore）限制并发数，防止 API 限流
- **结果聚合**：合并多个数据源的结果，去重和排序

### 4. 智能匹配与评分

使用多维度评分系统，确保返回最相关的结果：

```
总分 = 标题相似度 (70%) + 数据完整度 (30%) + 数据源优先级加成
```

- **标题相似度**：Levenshtein 编辑距离算法，容忍拼写差异
- **数据完整度**：有封面、简介、发行日期等字段的结果得分更高
- **数据源优先级**：DLsite (90) > IGDB (80) > TheGamesDB (70)

### 5. 可扩展架构

通过 Trait 定义接口，轻松扩展功能：

- **`GameDatabaseProvider`**：实现此 trait 即可添加新的数据源
- **`JsonOutput`**：为任何类型添加 JSON 导出能力
- **`GameMetadataFilter`**：自定义元数据过滤和筛选逻辑

### 6. 缓存与性能优化

- **结果缓存**：查询结果缓存 1 小时，避免重复 API 调用
- **并行扫描**：文件扫描使用 CPU 核心数的线程池
- **限流保护**：最多 5 个并发 API 请求，防止触发限流
- **滚动数组优化**：Levenshtein 算法使用 O(n) 空间复杂度而非 O(n²)

## Quick Start

### Scanning Local Game Directory

```rust
use gamebox::scan::GameScanner;

#[tokio::main]
async fn main() {
    // 创建游戏扫描器实例（同步操作，无需 await）
    let game_infos = GameScanner::new()
        // 添加 DLsite 数据源（适合日系游戏、视觉小说）
        .with_dlsite_provider().await
        // 添加 IGDB 数据源（适合欧美游戏、3A 大作）
        // 需要在 https://api-docs.igdb.com/ 注册获取凭证
        .with_igdb_provider(
            "your_client_id".to_string(),      // 你的 Twitch Client ID
            "your_client_secret".to_string()   // 你的 Twitch Client Secret
        ).await
        // 执行扫描，传入游戏目录路径
        .scan("D:/Games".to_string())
        .await;

    // 打印找到的游戏数量
    println!("Found {} games", game_infos.len());

    // 遍历所有游戏，打印标题和路径
    for game in game_infos {
        println!("- {} ({})", game.title, game.dir_path.display());
    }
}
```

### Searching Game Databases

```rust
use gamebox::scan::GameScanner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建扫描器并配置数据源
    let results = GameScanner::new()
        .with_dlsite_provider().await
        .with_igdb_provider(
            "your_client_id".to_string(),
            "your_client_secret".to_string()
        ).await
        // 搜索游戏名称，会在所有已注册的数据源中查询
        .search("Elden Ring".to_string())
        .await?;  // 返回 Result，需要处理错误

    // 遍历搜索结果
    for result in results {
        println!("Found: {} (Source: {}, Confidence: {:.2})",
            result.info.title.unwrap_or_default(),  // 游戏标题
            result.source,                          // 数据来源（DLsite/IGDB/TheGamesDB）
            result.confidence                       // 匹配置信度 (0.0-1.0)
        );
    }

    Ok(())
}
```

### Exporting Results to JSON

```rust
use gamebox::scan::GameScanner;
use gamebox::traits::JsonOutput;  // 导入 JSON 导出 trait

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 搜索游戏
    let results = GameScanner::new()
        .with_dlsite_provider().await
        .search("game name".to_string())
        .await?;

    // 导出到默认路径 (search_result.json)
    let path = results.out_json::<&str>(None)?;
    println!("Results saved to: {}", path);

    // 导出到自定义路径
    let scan_results = GameScanner::new()
        .with_dlsite_provider().await
        .scan("D:/Games".to_string())
        .await;

    // 指定文件名导出
    let custom_path = scan_results.out_json(Some("my_games.json"))?;
    println!("Scan results saved to: {}", custom_path);

    Ok(())
}
```

## Supported Database Providers

### DLsite
- **Priority**: 90 (Highest for Japanese games)
- **Best for**: Visual novels, Japanese RPGs, doujin games
- **No credentials required**

```rust
let scanner = GameScanner::new()
    .with_dlsite_provider().await;
```

### IGDB (Internet Game Database)
- **Priority**: 80
- **Best for**: Western games, AAA titles, indie games
- **Requires**: Twitch API credentials ([Get credentials](https://api-docs.igdb.com/#account-creation))

```rust
let scanner = GameScanner::new()
    .with_igdb_provider(
        "your_client_id".to_string(),
        "your_client_secret".to_string()
    ).await;
```

### TheGamesDB
- **Priority**: 70
- **Best for**: Classic games, retro games, multi-platform titles
- **No credentials required**

```rust
let scanner = GameScanner::new()
    .with_thegamesdb_provider().await;
```

### Custom Providers

你可以实现自己的游戏数据库提供者：

```rust
use async_trait::async_trait;
use gamebox::providers::GameDatabaseProvider;
use gamebox::models::game_meta_data::GameMetadata;
use std::sync::Arc;

// 定义自定义数据源
struct MyCustomProvider;

#[async_trait]
impl GameDatabaseProvider for MyCustomProvider {
    // 数据源名称
    fn name(&self) -> &str {
        "MyCustomDB"
    }

    // 搜索游戏（根据标题）
    async fn search(&self, title: &str) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error>> {
        // 在这里实现你的搜索逻辑
        // 例如：调用自定义 API、查询本地数据库等
        Ok(vec![])
    }

    // 根据 ID 获取游戏详情
    async fn get_by_id(&self, id: &str) -> Result<GameMetadata, Box<dyn std::error::Error>> {
        // 在这里实现获取详情的逻辑
        Ok(GameMetadata::default())
    }

    // 数据源优先级（数值越高优先级越高）
    fn priority(&self) -> u32 {
        50  // DLsite=90, IGDB=80, TheGamesDB=70
    }

    // 是否支持特定类型的游戏
    fn supports_game_type(&self, game_type: &str) -> bool {
        true  // 支持所有类型
    }
}

// 使用自定义数据源
let scanner = GameScanner::new()
    .with_provider(Arc::new(MyCustomProvider)).await;
```

## How It Works

### Scanning Process

1. **Parallel File Scanning** - Uses `ignore` crate with multi-threading to quickly scan directories for `.exe` files
2. **Intelligent Grouping** - Groups executables by their common parent directory to identify game root folders
3. **Pattern Matching** - Extracts game titles by removing version numbers, platform tags, and other noise
4. **Metadata Fetching** - Queries registered providers in parallel with rate limiting
5. **Confidence Scoring** - Ranks results based on title similarity, data completeness, and provider priority

### Confidence Scoring Algorithm

The confidence score (0.0 - 1.0) is calculated based on:

- **Title Similarity** (70%): Levenshtein distance between search query and result title
- **Data Completeness** (30%): Presence of metadata fields (cover, description, release date, etc.)
- **Provider Priority**: Higher priority providers get slight boost

### Caching Strategy

- Results are cached for 1 hour by default
- Cache key is the search query string
- Reduces API calls and improves performance for repeated queries

## Architecture

```
gamebox/
├── models/          # Data structures (GameInfo, GameMetadata)
├── providers/       # Database provider implementations
│   ├── dlsite_provider.rs
│   ├── igdb_provider.rs
│   └── thegamesdb_provider.rs
├── scan/            # Scanning logic
│   ├── scanner.rs   # Main GameScanner
│   ├── game_grouping.rs
│   ├── patterns.rs  # Regex patterns for title extraction
│   └── utils.rs
├── traits/          # Trait definitions
│   ├── game_metadata_filter.rs
│   └── json_output.rs
└── logger.rs        # Logging utilities
```

## Examples

See the [`examples/`](examples/) directory for more detailed examples:

- [`usage_example.rs`](examples/usage_example.rs) - Comprehensive usage examples

Run examples with:

```bash
cargo run --example usage_example
```

## Requirements

- Rust 2021 edition or later
- Tokio async runtime
- For IGDB provider: Twitch API credentials(If you need IGDB)

## Performance

- **Parallel Scanning**: Uses all available CPU cores for file scanning
- **Concurrent API Queries**: Up to 5 concurrent API requests (configurable)
- **Smart Caching**: Reduces redundant API calls
- **Efficient Algorithms**: Optimized Levenshtein distance calculation with rolling arrays

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [dlsite-rs](https://github.com/ozonezone/dlsite-rs) - DLsite API client
- [IGDB API](https://api-docs.igdb.com/) - Internet Game Database
- [TheGamesDB](https://thegamesdb.net/) - Classic game database

## Roadmap

- [ ] Add more database providers (Steam, GOG, etc.)
- [ ] Support for non-Windows platforms
- [ ] GUI application
- [ ] Plugin system for custom metadata enrichment
- [ ] Database export/import functionality
- [ ] Game launcher integration

