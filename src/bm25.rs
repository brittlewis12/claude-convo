use std::collections::HashMap;

/// BM25 scorer for ranking text documents
pub struct BM25 {
    /// Average document length
    avg_doc_length: f64,
    /// Total number of documents
    doc_count: usize,
    /// Document frequencies for each term
    doc_frequencies: HashMap<String, usize>,
    /// k1 parameter (controls term frequency saturation)
    k1: f64,
    /// b parameter (controls length normalization)
    b: f64,
}

impl BM25 {
    /// Create a new BM25 scorer from a corpus of documents
    pub fn new(documents: &[String], k1: f64, b: f64) -> Self {
        let doc_count = documents.len();
        let mut total_length = 0;
        let mut doc_frequencies = HashMap::new();

        // Calculate document frequencies and total length
        for doc in documents {
            let tokens = tokenize(doc);
            total_length += tokens.len();

            // Track unique terms in this document
            let mut seen = HashMap::new();
            for token in tokens {
                // insert() returns None if the key wasn't present
                if seen.insert(token.clone(), true).is_none() {
                    *doc_frequencies.entry(token).or_insert(0) += 1;
                }
            }
        }

        let avg_doc_length = if doc_count > 0 {
            total_length as f64 / doc_count as f64
        } else {
            0.0
        };

        BM25 {
            avg_doc_length,
            doc_count,
            doc_frequencies,
            k1,
            b,
        }
    }

    /// Score a single document against a query
    pub fn score(&self, query: &str, document: &str) -> f64 {
        let query_terms = tokenize(query);
        let doc_terms = tokenize(document);
        let doc_length = doc_terms.len() as f64;

        // Count term frequencies in document
        let mut term_freqs = HashMap::new();
        for term in &doc_terms {
            *term_freqs.entry(term.clone()).or_insert(0) += 1;
        }

        let mut score = 0.0;

        for query_term in query_terms {
            if let Some(tf) = term_freqs.get(&query_term) {
                let tf = *tf as f64;

                // IDF calculation
                let df = self.doc_frequencies.get(&query_term).unwrap_or(&0);
                let idf = ((self.doc_count as f64 - *df as f64 + 0.5) / (*df as f64 + 0.5)).ln();

                // BM25 formula
                let normalized_tf = (tf * (self.k1 + 1.0))
                    / (tf + self.k1 * (1.0 - self.b + self.b * (doc_length / self.avg_doc_length)));

                score += idf * normalized_tf;
            }
        }

        score
    }
}

/// Simple tokenizer - splits on whitespace and converts to lowercase
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_basic() {
        let docs = vec![
            "the quick brown fox jumps over the lazy dog".to_string(),
            "the lazy dog sleeps all day".to_string(),
            "the brown fox hunts at night".to_string(),
            "cats are completely different animals".to_string(),
            "programming in rust is fun".to_string(),
        ];

        let bm25 = BM25::new(&docs, 1.2, 0.75);

        // Test 1: Document containing both query terms should score higher than one with neither
        let score_both = bm25.score("brown fox", &docs[0]); // has both terms
        let score_neither = bm25.score("brown fox", &docs[4]); // has neither term
        assert!(score_both > score_neither);

        // Test 2: Exact term matching
        let score_fox1 = bm25.score("fox", &docs[0]); // has "fox"
        let score_fox2 = bm25.score("fox", &docs[1]); // no "fox"
        assert!(score_fox1 > score_fox2);

        // Test 3: Multiple occurrences and document length normalization
        let score_dog1 = bm25.score("dog", &docs[0]); // "dog" in longer doc
        let score_dog2 = bm25.score("dog", &docs[1]); // "dog" in shorter doc
                                                      // Both have "dog" once, but doc2 is shorter so should score higher
        assert!(score_dog2 > score_dog1);
    }
}
