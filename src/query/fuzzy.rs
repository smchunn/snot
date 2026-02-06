use std::collections::HashSet;

/// Calculate trigram similarity between two strings.
/// Returns a value between 0.0 (no match) and 1.0 (identical).
pub fn trigram_similarity(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    let a_trigrams = trigrams(&a_lower);
    let b_trigrams = trigrams(&b_lower);

    if a_trigrams.is_empty() && b_trigrams.is_empty() {
        return 1.0;
    }
    if a_trigrams.is_empty() || b_trigrams.is_empty() {
        return 0.0;
    }

    let intersection = a_trigrams.intersection(&b_trigrams).count();
    let union = a_trigrams.union(&b_trigrams).count();

    intersection as f64 / union as f64
}

/// Generate trigrams for a string.
/// Pads with spaces at the start and end for better matching.
fn trigrams(s: &str) -> HashSet<String> {
    let padded = format!("  {}  ", s);
    let chars: Vec<char> = padded.chars().collect();

    if chars.len() < 3 {
        return HashSet::new();
    }

    (0..chars.len() - 2)
        .map(|i| chars[i..i + 3].iter().collect::<String>())
        .collect()
}

/// Default similarity threshold for fuzzy matching.
pub const FUZZY_THRESHOLD: f64 = 0.2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_strings() {
        assert!((trigram_similarity("hello", "hello") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_similar_strings() {
        let sim = trigram_similarity("meeting", "meting");
        assert!(sim > 0.3, "Expected > 0.3, got {}", sim);
    }

    #[test]
    fn test_different_strings() {
        let sim = trigram_similarity("hello", "world");
        assert!(sim < 0.2, "Expected < 0.2, got {}", sim);
    }

    #[test]
    fn test_case_insensitive() {
        assert!((trigram_similarity("Hello", "hello") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_strings() {
        assert!((trigram_similarity("", "") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_one_empty() {
        assert!((trigram_similarity("hello", "")).abs() < f64::EPSILON);
    }

    #[test]
    fn test_typo_detection() {
        // Common typo: transposition
        let sim = trigram_similarity("project", "porject");
        assert!(sim > FUZZY_THRESHOLD, "Expected > threshold, got {}", sim);
    }
}
