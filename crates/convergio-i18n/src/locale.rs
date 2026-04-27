//! Locale enum + resolution from CLI flag / env vars.

use crate::error::{I18nError, Result};
use serde::{Deserialize, Serialize};

/// Locales we ship with first-class support.
///
/// Adding a new locale: add a variant here, a directory under
/// `locales/<tag>/`, an entry in [`Locale::tag`] /
/// [`Locale::from_tag`], and a translation of every key. CI
/// `I18nCoverageGate` enforces full key coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// English — fallback.
    #[default]
    En,
    /// Italian.
    It,
}

impl Locale {
    /// All locales we ship.
    pub const ALL: &'static [Locale] = &[Locale::En, Locale::It];

    /// IETF-ish short tag (no region).
    pub fn tag(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::It => "it",
        }
    }

    /// Parse a short tag. Accepts `en`, `en-US`, `en_GB`,
    /// `it`, `it-IT`, etc. — only the first 2 chars (lowercased) are
    /// looked at.
    pub fn from_tag(s: &str) -> Result<Self> {
        let head: String = s.chars().take(2).flat_map(char::to_lowercase).collect();
        match head.as_str() {
            "en" => Ok(Self::En),
            "it" => Ok(Self::It),
            _ => Err(I18nError::UnsupportedLocale(s.to_string())),
        }
    }
}

// Default is derived via `#[default]` on the `En` variant.

/// Resolve which locale to use from the supplied override and the
/// process environment.
///
/// Order:
/// 1. `explicit` (e.g. `--lang it`)
/// 2. `CONVERGIO_LANG`
/// 3. `LANG` / `LC_ALL`
/// 4. Default `en`
pub fn detect_locale(explicit: Option<&str>) -> Locale {
    if let Some(s) = explicit {
        if let Ok(loc) = Locale::from_tag(s) {
            return loc;
        }
    }
    for var in ["CONVERGIO_LANG", "LC_ALL", "LANG"] {
        if let Ok(v) = std::env::var(var) {
            // `it_IT.UTF-8`, `en_US`, `it`, etc.
            let first = v.split(['_', '-', '.', '@']).next().unwrap_or("");
            if let Ok(loc) = Locale::from_tag(first) {
                return loc;
            }
        }
    }
    Locale::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_tag_short() {
        assert_eq!(Locale::from_tag("en").unwrap(), Locale::En);
        assert_eq!(Locale::from_tag("it").unwrap(), Locale::It);
    }

    #[test]
    fn from_tag_long() {
        assert_eq!(Locale::from_tag("en-US").unwrap(), Locale::En);
        assert_eq!(Locale::from_tag("it-IT").unwrap(), Locale::It);
        assert_eq!(Locale::from_tag("it_IT.UTF-8").unwrap(), Locale::It);
    }

    #[test]
    fn from_tag_unknown_errors() {
        assert!(Locale::from_tag("zz").is_err());
    }

    #[test]
    fn explicit_wins_over_env() {
        // Even if LANG were set, `explicit` overrides.
        let loc = detect_locale(Some("it"));
        assert_eq!(loc, Locale::It);
    }

    #[test]
    fn unsupported_explicit_falls_through() {
        // An invalid `--lang zz` should not crash; it falls through to
        // env / default.
        let loc = detect_locale(Some("zz"));
        // It cannot return UnsupportedLocale here — it falls through.
        // We assert it's one of the supported ones, exact value depends
        // on CI environment.
        assert!(Locale::ALL.contains(&loc));
    }
}
