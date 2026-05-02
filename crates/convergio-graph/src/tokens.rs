//! Tokeniser + stopword list shared by the query layer.
//!
//! Split out of [`super::query`] to honour the 300-line per-file cap.

use std::collections::BTreeSet;

/// Maximum task text characters considered by the query tokeniser.
pub const MAX_QUERY_INPUT_CHARS: usize = 16_384;

/// Maximum distinct query tokens emitted for graph ranking.
pub const MAX_QUERY_TOKENS: usize = 64;

/// Tokenise the task text. Lowercases, splits on non-alphanumeric,
/// drops stopwords + tokens shorter than 3 chars, deduplicates, and
/// caps input/tokens so one oversized task cannot fan out unbounded
/// graph queries.
pub fn tokenise(text: &str) -> Vec<String> {
    let limited: String = text.chars().take(MAX_QUERY_INPUT_CHARS).collect();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for raw in limited.split(|c: char| !c.is_alphanumeric()) {
        if raw.len() < 3 {
            continue;
        }
        let lc = raw.to_ascii_lowercase();
        if STOPWORDS.contains(&lc.as_str()) {
            continue;
        }
        if seen.insert(lc.clone()) {
            out.push(lc);
            if out.len() >= MAX_QUERY_TOKENS {
                break;
            }
        }
    }
    out
}

/// Tokens we never want as queries — too generic to be informative.
/// Includes English stopwords plus high-frequency code-domain words
/// ("command", "file", "code", "test", "task") that match nearly
/// every module name and produce flat tied scores.
static STOPWORDS: &[&str] = &[
    // English
    "the",
    "and",
    "for",
    "with",
    "into",
    "from",
    "this",
    "that",
    "user",
    "via",
    "are",
    "not",
    "but",
    "all",
    "any",
    "you",
    "your",
    "our",
    "out",
    "was",
    "will",
    "should",
    "must",
    "may",
    "can",
    "now",
    "new",
    "old",
    "yes",
    "see",
    "non",
    "let",
    "one",
    "two",
    "have",
    "has",
    "had",
    "been",
    "being",
    "they",
    "them",
    "their",
    "than",
    "then",
    "where",
    "when",
    "what",
    "which",
    "who",
    "how",
    "why",
    "did",
    "does",
    "done",
    "even",
    "such",
    "some",
    // Code-domain noise
    "task",
    "code",
    "file",
    "files",
    "test",
    "tests",
    "command",
    "commands",
    "module",
    "modules",
    "use",
    "uses",
    "used",
    "run",
    "runs",
    "ran",
    "running",
    "src",
    "lib",
    "rs",
    "md",
    "yml",
    "yaml",
    "json",
    "toml",
    "scripts",
    "script",
    "repo",
    "git",
    // Frequent meta words from task descriptions
    "acceptance",
    "available",
    "automatically",
    "tracks",
    "needs",
    "without",
    "before",
    "after",
    "until",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_stopwords_and_short() {
        let toks = tokenise("Add the convergio-graph crate (syn-based) for Tier-3 retrieval");
        assert!(toks.contains(&"convergio".to_string()));
        assert!(toks.contains(&"graph".to_string()));
        assert!(toks.contains(&"crate".to_string()));
        assert!(toks.contains(&"syn".to_string()));
        assert!(toks.contains(&"based".to_string()));
        assert!(toks.contains(&"retrieval".to_string()));
        assert!(!toks.contains(&"the".to_string()));
        assert!(!toks.contains(&"for".to_string()));
        assert!(!toks.contains(&"task".to_string()));
        assert!(!toks.iter().any(|t| t.len() < 3));
    }

    #[test]
    fn dedups() {
        let toks = tokenise("foo foo bar foo");
        assert_eq!(toks, vec!["foo", "bar"]);
    }

    #[test]
    fn caps_distinct_tokens() {
        let input = (0..(MAX_QUERY_TOKENS + 20))
            .map(|i| format!("tok{i:03}"))
            .collect::<Vec<_>>()
            .join(" ");
        let toks = tokenise(&input);
        assert_eq!(toks.len(), MAX_QUERY_TOKENS);
        assert_eq!(toks.first(), Some(&"tok000".to_string()));
        assert_eq!(
            toks.last(),
            Some(&format!("tok{:03}", MAX_QUERY_TOKENS - 1))
        );
    }

    #[test]
    fn caps_input_before_tokenising() {
        let mut input = "a".repeat(MAX_QUERY_INPUT_CHARS);
        input.push_str(" hidden_token");
        let toks = tokenise(&input);
        assert!(!toks.contains(&"hidden".to_string()));
        assert!(!toks.contains(&"token".to_string()));
    }
}
