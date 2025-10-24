use std::collections::HashMap;

/// Extracts key phrases from text using lightweight NLP
/// Returns top N most relevant phrases without heavy ML dependencies
#[allow(dead_code)]
pub fn extract_key_phrases(text: &str, max_phrases: usize) -> Vec<String> {
    // Tokenize and remove stop words
    let words: Vec<&str> = text
        .split_whitespace()
        .filter(|w| !is_stop_word(w))
        .collect();

    if words.is_empty() {
        return Vec::new();
    }

    // Generate n-grams (1-3 words)
    let mut ngrams = Vec::new();

    for i in 0..words.len() {
        // Unigrams
        ngrams.push(words[i].to_string());

        // Bigrams
        if i + 1 < words.len() {
            ngrams.push(format!("{} {}", words[i], words[i + 1]));
        }

        // Trigrams
        if i + 2 < words.len() {
            ngrams.push(format!("{} {} {}", words[i], words[i + 1], words[i + 2]));
        }
    }

    // Score by frequency and position (earlier = better)
    let mut scored: HashMap<String, f32> = HashMap::new();

    for (idx, ngram) in ngrams.iter().enumerate() {
        // Position score: earlier text weighted higher
        let position_score = 1.0 / (1.0 + idx as f32 * 0.05);

        // Length bonus: prefer multi-word phrases
        let word_count = ngram.split_whitespace().count();
        let length_bonus = word_count as f32 * 0.3;

        // Combine scores
        let score = position_score + length_bonus;

        *scored.entry(ngram.clone()).or_insert(0.0) += score;
    }

    // Get top N phrases
    let mut phrases: Vec<_> = scored.into_iter().collect();
    phrases.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    phrases.truncate(max_phrases);

    phrases.into_iter().map(|(phrase, _)| phrase).collect()
}

/// Checks if a word is a common stop word
#[allow(dead_code)]
fn is_stop_word(word: &str) -> bool {
    let lower = word.to_lowercase();

    let stop_words = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at",
        "to", "for", "of", "with", "by", "from", "as", "is", "was",
        "are", "were", "been", "be", "have", "has", "had", "do", "does",
        "did", "will", "would", "could", "should", "may", "might", "must",
        "can", "this", "that", "these", "those", "i", "you", "he", "she",
        "it", "we", "they", "what", "which", "who", "when", "where", "why",
        "how",
    ];

    stop_words.contains(&lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_key_phrases_basic() {
        let text = "Quarterly Sales Report for Q3 2023 showing revenue growth";
        let phrases = extract_key_phrases(text, 3);

        assert!(!phrases.is_empty());
        // Should prioritize multi-word phrases and earlier text
        assert!(phrases[0].contains("Quarterly") || phrases[0].contains("Sales"));
    }

    #[test]
    fn test_extract_key_phrases_filters_stop_words() {
        let text = "The report is about the quarterly sales and the revenue";
        let phrases = extract_key_phrases(text, 5);

        // Should not include pure stop words
        assert!(!phrases.contains(&"the".to_string()));
        assert!(!phrases.contains(&"is".to_string()));
        assert!(!phrases.contains(&"and".to_string()));
    }

    #[test]
    fn test_extract_key_phrases_prioritizes_bigrams() {
        let text = "Machine Learning Applications in Healthcare Systems";
        let phrases = extract_key_phrases(text, 3);

        // Should include bigrams/trigrams
        let has_multi_word = phrases.iter().any(|p| p.split_whitespace().count() > 1);
        assert!(has_multi_word, "Should include multi-word phrases");
    }

    #[test]
    fn test_extract_key_phrases_position_weighting() {
        let text = "Important Document about routine maintenance";
        let phrases = extract_key_phrases(text, 2);

        // "Important" and "Document" should rank higher (earlier position)
        assert!(
            phrases.iter().any(|p| p.contains("Important") || p.contains("Document")),
            "Should prioritize earlier text"
        );
    }

    #[test]
    fn test_extract_key_phrases_empty_text() {
        let text = "";
        let phrases = extract_key_phrases(text, 3);
        assert!(phrases.is_empty());
    }

    #[test]
    fn test_extract_key_phrases_only_stop_words() {
        let text = "the and or but with";
        let phrases = extract_key_phrases(text, 3);
        assert!(phrases.is_empty());
    }

    #[test]
    fn test_is_stop_word() {
        assert!(is_stop_word("the"));
        assert!(is_stop_word("THE")); // case insensitive
        assert!(is_stop_word("and"));
        assert!(is_stop_word("with"));

        assert!(!is_stop_word("document"));
        assert!(!is_stop_word("important"));
        assert!(!is_stop_word("sales"));
    }

    #[test]
    fn test_extract_key_phrases_respects_limit() {
        let text = "One Two Three Four Five Six Seven Eight Nine Ten";
        let phrases = extract_key_phrases(text, 3);

        assert_eq!(phrases.len(), 3, "Should return exactly max_phrases");
    }
}
