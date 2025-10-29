# GameBox

[![Crates.io](https://img.shields.io/crates/v/gamebox.svg)](https://crates.io/crates/gamebox)
[![Documentation](https://docs.rs/gamebox/badge.svg)](https://docs.rs/gamebox)
[![License](https://img.shields.io/crates/l/gamebox.svg)](LICENSE)

A powerful Rust library for scanning local game directories and fetching game metadata from multiple database providers.

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

## Quick Start

### Scanning Local Game Directory

```rust
use gamebox::scan::GameScanner;

#[tokio::main]
async fn main() {
    let game_infos = GameScanner::new()
        .with_dlsite_provider().await
        .with_igdb_provider(
            "your_client_id".to_string(),
            "your_client_secret".to_string()
        ).await
        .scan("D:/Games".to_string())
        .await;

    println!("Found {} games", game_infos.len());
    
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
    let results = GameScanner::new()
        .with_dlsite_provider().await
        .with_igdb_provider(
            "your_client_id".to_string(),
            "your_client_secret".to_string()
        ).await
        .search("Elden Ring".to_string())
        .await?;

    for result in results {
        println!("Found: {} (Source: {}, Confidence: {:.2})",
            result.info.title.unwrap_or_default(),
            result.source,
            result.confidence
        );
    }
    
    Ok(())
}
```

### Exporting Results to JSON

```rust
use gamebox::scan::GameScanner;
use gamebox::traits::JsonOutput;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = GameScanner::new()
        .with_dlsite_provider().await
        .search("game name".to_string())
        .await?;

    // Export to default path (search_result.json)
    let path = results.out_json::<&str>(None)?;
    println!("Results saved to: {}", path);

    // Export to custom path
    let scan_results = GameScanner::new()
        .with_dlsite_provider().await
        .scan("D:/Games".to_string())
        .await;
    
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

You can implement your own game database provider:

```rust
use async_trait::async_trait;
use gamebox::providers::GameDatabaseProvider;
use gamebox::models::game_meta_data::GameMetadata;

struct MyCustomProvider;

#[async_trait]
impl GameDatabaseProvider for MyCustomProvider {
    fn name(&self) -> &str {
        "MyCustomDB"
    }

    async fn search(&self, title: &str) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error>> {
        // Your implementation
        Ok(vec![])
    }

    async fn get_by_id(&self, id: &str) -> Result<GameMetadata, Box<dyn std::error::Error>> {
        // Your implementation
        Ok(GameMetadata::default())
    }

    fn priority(&self) -> u32 {
        50
    }

    fn supports_game_type(&self, game_type: &str) -> bool {
        true
    }
}

// Use it
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
- For IGDB provider: Twitch API credentials

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

