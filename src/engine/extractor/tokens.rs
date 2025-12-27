use std::{collections::HashMap, io::{BufRead, BufReader, Read}};

pub const MIN_TOKEN_LENGTH: usize = 3;

#[derive(Debug, Clone)]
pub struct TextTokensDistribution {
    distribution: HashMap<String, usize>,
}

impl Default for TextTokensDistribution {
    fn default() -> Self {
        TextTokensDistribution {
            distribution: HashMap::new(),
        }
    }
}

impl TextTokensDistribution {
    pub fn get_tokens(line: &str) -> impl Iterator<Item = &str> {
        line.split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= MIN_TOKEN_LENGTH)
    }

    pub fn from_buffer(reader: BufReader<impl Read>) -> Self {
        let mut dist = TextTokensDistribution::default();

        for line in reader.lines() {
            if let Ok(line) = line {
                for word in Self::get_tokens(&line) {
                    dist.add_word(word);
                }
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

    pub fn export_string(&self, n: usize) -> String {
        let top_words = self.top_n(n);
        top_words
            .into_iter()
            .map(|(word, _)| word)
            .collect::<Vec<String>>()
            .join(" ")
    }
}