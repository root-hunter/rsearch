use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
};

use tracing::warn;

use crate::{
    engine::extractor::formats::{DataExtracted, FileExtractor},
    entities::document::Document,
};

//const LOG_TARGET: &str = "extractor_text";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextExtractor;

pub struct TextDistribution {
    distribution: HashMap<String, usize>,
}

impl Default for TextDistribution {
    fn default() -> Self {
        TextDistribution {
            distribution: HashMap::new(),
        }
    }
}

impl TextDistribution {
    pub fn get_tokens(line: &str) -> impl Iterator<Item = &str> {
        line.split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 3)
    }

    pub fn from_buffer(reader: BufReader<impl Read>) -> Self {
        let mut dist = TextDistribution::default();

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

impl FileExtractor for TextExtractor {
    fn extract(&self, document: Document) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        let file = File::open(document.get_path())?;
        let reader = BufReader::new(file);

        let dist = TextDistribution::from_buffer(reader);
        let content = dist.export_string(200);

        Ok(DataExtracted::Text(content))
    }
}
