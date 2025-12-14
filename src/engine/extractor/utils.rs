use std::{collections::{BTreeMap, HashMap}, env};

use once_cell::sync::Lazy;
use tracing::info;

pub const LOG_TARGET: &str = "extractor_utils";

static EXTRACTOR_DISTRIBUTION_MAX_WORDS: Lazy<usize> = Lazy::new(|| {
    env::var("EXTRACTOR_DISTRIBUTION_MAX_WORDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200)
});

static EXTRACTOR_MIN_WORD_LENGTH: Lazy<usize> = Lazy::new(|| {
    env::var("EXTRACTOR_MIN_WORD_LENGTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3)
});

pub fn text_normalize(text: &str) -> String {
    let s: String = text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();

    s.to_lowercase()
}

pub fn text_extract_distribution(text: String) -> Vec<String> {
    let mut distribution: BTreeMap<String, usize> = std::collections::BTreeMap::new();

    let text = text_normalize(&text);

    for word in text.split(' ') {
        let word = word.replace('\n', "").replace('\r', "");

        if word.len() < *EXTRACTOR_MIN_WORD_LENGTH {
            continue;
        }
        
        *distribution.entry(word.into()).or_insert(0) += 1;
    }

    let mut words: Vec<_> = distribution.into_iter().collect();
    words.sort_by(|a, b| b.1.cmp(&a.1));
    words.truncate(*EXTRACTOR_DISTRIBUTION_MAX_WORDS);

    words.iter().map(|e| e.0.clone()).collect()
}

pub fn build_text_content(text: String) -> String {
    let distribtuion = text_extract_distribution(text);

    distribtuion.join(" ")
}