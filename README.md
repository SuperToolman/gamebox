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

## 功能特性

- 🔍 **智能游戏扫描** - 自动扫描目录中的游戏可执行文件，智能分组到游戏根目录
- 🌐 **多数据源支持** - 支持 DLsite、IGDB 和 TheGamesDB，可扩展的数据源系统
- 🎯 **智能匹配** - 使用 Levenshtein 距离算法进行模糊标题匹配，带置信度评分
- ⚡ **高性能** - 多线程并行文件扫描和并发 API 查询
- 📦 **灵活的 API** - 流式构建器模式，支持方法链式调用，配置简单
- 💾 **智能缓存** - 内置结果缓存，可配置 TTL（默认 1 小时）
- 🚦 **限流保护** - 自动 API 限流，防止触发数据源限制
- 📊 **JSON 导出** - 将扫描和搜索结果导出为 JSON 格式
- 🔧 **可扩展** - 轻松添加自定义游戏数据库提供者

## 安装

在你的 `Cargo.toml` 中添加：

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

## 快速入门

### 扫描本地游戏目录

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

### 搜索游戏数据库

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

### 导出结果为 JSON

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

## 支持的数据库提供者

### DLsite
- **优先级**: 90（日系游戏最高优先级）
- **适用于**: 视觉小说、日系 RPG、同人游戏
- **无需凭证**

```rust
let scanner = GameScanner::new()
    .with_dlsite_provider().await;
```

### IGDB (互联网游戏数据库)
- **优先级**: 80
- **适用于**: 欧美游戏、3A 大作、独立游戏
- **需要**: Twitch API 凭证 ([获取凭证](https://api-docs.igdb.com/#account-creation))

```rust
let scanner = GameScanner::new()
    .with_igdb_provider(
        "your_client_id".to_string(),
        "your_client_secret".to_string()
    ).await;
```

### TheGamesDB
- **优先级**: 70
- **适用于**: 经典游戏、复古游戏、多平台游戏
- **无需凭证**

```rust
let scanner = GameScanner::new()
    .with_thegamesdb_provider().await;
```

### 自定义数据源

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

## 工作原理

### 扫描流程

1. **并行文件扫描** - 使用 `ignore` crate 配合多线程快速扫描目录中的 `.exe` 文件
2. **智能分组** - 根据可执行文件的公共父目录进行分组，识别游戏根文件夹
3. **模式匹配** - 通过移除版本号、平台标签等噪音信息提取游戏标题
4. **元数据获取** - 并行查询已注册的数据源，带限流保护
5. **置信度评分** - 根据标题相似度、数据完整度和数据源优先级对结果排序

### 置信度评分算法

置信度分数（0.0 - 1.0）基于以下因素计算：

- **标题相似度** (70%)：搜索查询与结果标题之间的 Levenshtein 距离
- **数据完整度** (30%)：元数据字段的存在性（封面、简介、发行日期等）
- **数据源优先级**：优先级更高的数据源获得轻微加成

### 缓存策略

- 默认缓存结果 1 小时
- 缓存键为搜索查询字符串
- 减少 API 调用，提高重复查询的性能

## 项目架构

```
gamebox/
├── models/          # 数据结构 (GameInfo, GameMetadata)
├── providers/       # 数据库提供者实现
│   ├── dlsite_provider.rs
│   ├── igdb_provider.rs
│   └── thegamesdb_provider.rs
├── scan/            # 扫描逻辑
│   ├── scanner.rs   # 主扫描器 GameScanner
│   ├── game_grouping.rs
│   ├── patterns.rs  # 标题提取的正则表达式模式
│   └── utils.rs
├── traits/          # Trait 定义
│   ├── game_metadata_filter.rs
│   └── json_output.rs
└── logger.rs        # 日志工具
```

## 示例

查看 [`examples/`](examples/) 目录获取更详细的示例：

- [`usage_example.rs`](examples/usage_example.rs) - 综合使用示例

运行示例：

```bash
cargo run --example usage_example
```

## 环境要求

- Rust 2021 edition 或更高版本
- Tokio 异步运行时
- IGDB 数据源需要：Twitch API 凭证（如果需要使用 IGDB）

## 性能

- **并行扫描**：使用所有可用 CPU 核心进行文件扫描
- **并发 API 查询**：最多 5 个并发 API 请求（可配置）
- **智能缓存**：减少冗余 API 调用
- **高效算法**：使用滚动数组优化的 Levenshtein 距离计算

## 贡献

欢迎贡献！请随时提交 Pull Request。

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [dlsite-rs](https://github.com/ozonezone/dlsite-rs) - DLsite API 客户端
- [IGDB API](https://api-docs.igdb.com/) - 互联网游戏数据库
- [TheGamesDB](https://thegamesdb.net/) - 经典游戏数据库

## 开发路线图

- [ ] 添加更多数据库提供者（Steam、GOG 等）
- [ ] 支持非 Windows 平台
- [ ] GUI 应用程序
- [ ] 插件系统用于自定义元数据增强
- [ ] 数据库导出/导入功能
- [ ] 游戏启动器集成

