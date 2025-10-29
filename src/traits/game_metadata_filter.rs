use std::cmp::Ordering;
use crate::models::game_meta_data::GameMetadata;

/// 实现查询多内容匹配值最大的游戏元数据
pub trait GameMetadataFilter {
    /// 找到标题匹配值最高的项
    fn find_best_match(self, query: &str) -> Option<GameMetadata>;
    /// 找到匹配值最高的多个项
    fn find_best_matches(self, query: &str, limit: usize) -> Vec<GameMetadata>;
}

impl GameMetadataFilter for Vec<GameMetadata> {
    fn find_best_match(self, query: &str) -> Option<GameMetadata> {
        self.find_best_matches(query, 1).into_iter().next()
    }

    fn find_best_matches(self, query: &str, limit: usize) -> Vec<GameMetadata> {
        let mut scored_games: Vec<(GameMetadata, f64)> = self
            .into_iter()
            .filter_map(|game| {
                game.title.clone().map(|title| (game, title))
            })
            .map(|(game, title)| {
                let score = calculate_match_score(query, &title);
                (game, score)
            })
            .collect();

        // 按匹配分数降序排序
        scored_games.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal)
        });

        scored_games
            .into_iter()
            .take(limit)
            .map(|(game, _)| game)
            .collect()
    }
}

/// 计算匹配分数（0.0 ~ 1.0）
fn calculate_match_score(query: &str, title: &str) -> f64 {
    let query = query.trim().to_lowercase();
    let title = title.trim().to_lowercase();

    if query.is_empty() || title.is_empty() {
        return 0.0;
    }

    // 1. 完全匹配检查
    if title == query {
        return 1.0;
    }

    // 2. 包含关系检查
    let contains_score = if title.contains(&query) {
        0.9 + (query.len() as f64 / title.len() as f64) * 0.1
    } else if query.contains(&title) {
        0.8
    } else {
        0.0
    };

    if contains_score > 0.0 {
        return contains_score;
    }

    // 3. 编辑距离 + 长度惩罚
    let edit_distance = levenshtein_distance(&query, &title);
    let max_len = query.len().max(title.len()) as f64;
    let distance_score = 1.0 - (edit_distance as f64 / max_len);

    // 4. 长度相似度惩罚（避免太长或太短的标题）
    let length_penalty = calculate_length_penalty(query.len(), title.len());

    distance_score * length_penalty
}

/// 计算Levenshtein编辑距离（优化版：空间复杂度 O(m) 而非 O(n*m)）
/// 使用滚动数组技术，只保留两行数据
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // 边界情况
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    // 使用两个一维数组代替二维矩阵（滚动数组技术）
    let mut prev_row = vec![0; b_len + 1];
    let mut curr_row = vec![0; b_len + 1];

    // 初始化第一行
    for j in 0..=b_len {
        prev_row[j] = j;
    }

    // 逐行计算
    for i in 1..=a_len {
        curr_row[0] = i; // 每行的第一列

        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr_row[j] = (prev_row[j] + 1)           // 删除
                .min(curr_row[j - 1] + 1)             // 插入
                .min(prev_row[j - 1] + cost);         // 替换
        }

        // 交换行（避免内存分配）
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// 计算长度惩罚因子（0.0 ~ 1.0）
fn calculate_length_penalty(query_len: usize, title_len: usize) -> f64 {
    let ratio = if query_len > title_len {
        title_len as f64 / query_len as f64
    } else {
        query_len as f64 / title_len as f64
    };

    // 长度差异越大，惩罚越大
    match ratio {
        r if r >= 0.8 => 1.0,    // 长度很接近，不惩罚
        r if r >= 0.6 => 0.8,    // 长度有一定差异
        r if r >= 0.4 => 0.5,    // 长度差异较大
        _ => 0.2,                // 长度差异很大
    }
}

