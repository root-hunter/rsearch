use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use once_cell::sync::Lazy;

use crate::engine::extractor::constants;

pub static EXTRACTOR_TOKENS_MIN_LENGTH: Lazy<usize> = Lazy::new(|| {
    std::env::var("EXTRACTOR_TOKENS_MIN_LENGTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(constants::DEFAULT_TOKENS_MIN_LENGTH)
});

#[derive(Debug, Clone, Default)]
pub struct TextTokensDistribution {
    distribution: HashMap<String, usize>,
}

impl TextTokensDistribution {
    pub fn get_tokens(line: &str) -> impl Iterator<Item = &str> {
        line.split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= *EXTRACTOR_TOKENS_MIN_LENGTH)
    }

    pub fn from_buffer(reader: BufReader<impl Read>) -> Self {
        let mut dist = TextTokensDistribution::default();

        for line in reader.lines().map_while(Result::ok) {
            for word in Self::get_tokens(&line) {
                dist.add_word(word);
            }
        }

        dist
    }

    pub fn add_word(&mut self, word: &str) {
        let word = word.to_lowercase();
        *self.distribution.entry(word).or_insert(0) += 1;
    }

    pub fn top_n(&self, n: usize) -> Vec<(String, usize)> {
        let mut words: Vec<_> = self.distribution.iter().collect();
        words.sort_by(|a, b| b.1.cmp(a.1));
        words
            .into_iter()
            .take(n)
            .map(|(word, &count)| (word.clone(), count))
            .collect()
    }

    pub fn export_string_nth(&self, n: usize) -> String {
        let top_words = self.top_n(n);
        top_words
            .into_iter()
            .map(|(word, _)| word)
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn export_string(&self) -> String {
        self.export_string_nth(self.distribution.len())
    }
}
