//! Coverage gate: every message key in the English bundle must exist
//! in every other shipped locale. This is the i18n version of "every
//! pub fn must have a test" — non-negotiable per P5.

use convergio_i18n::{Bundle, Locale};

/// Re-parse the .ftl files at test time and extract every message key.
fn keys(locale: Locale) -> Vec<String> {
    let src = match locale {
        Locale::En => include_str!("../locales/en/main.ftl"),
        Locale::It => include_str!("../locales/it/main.ftl"),
    };
    src.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            // A message line looks like `key = ...`. We treat any
            // line containing ` = ` whose left-hand side is an
            // identifier as a message key.
            line.split_once('=').and_then(|(left, _)| {
                let k = left.trim();
                if k.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                    && !k.is_empty()
                {
                    Some(k.to_string())
                } else {
                    None
                }
            })
        })
        .collect()
}

#[test]
fn every_locale_loads() {
    for &loc in Locale::ALL {
        Bundle::new(loc).unwrap_or_else(|e| panic!("bundle for {loc:?} failed to load: {e}"));
    }
}

#[test]
fn italian_has_every_english_key() {
    let en_keys = keys(Locale::En);
    let it_keys = keys(Locale::It);
    let missing: Vec<&String> = en_keys.iter().filter(|k| !it_keys.contains(k)).collect();
    assert!(
        missing.is_empty(),
        "Italian bundle missing keys: {missing:?}"
    );
}

#[test]
fn english_has_every_italian_key() {
    let en_keys = keys(Locale::En);
    let it_keys = keys(Locale::It);
    let missing: Vec<&String> = it_keys.iter().filter(|k| !en_keys.contains(k)).collect();
    assert!(
        missing.is_empty(),
        "English bundle missing keys: {missing:?}"
    );
}

#[test]
fn every_locale_resolves_known_keys() {
    // Sanity: take a handful of keys from the English file, format
    // them in every locale, and assert no crash and no "key returned
    // verbatim" (which would mean the key is missing).
    let probe_keys = [
        "ok",
        "plan-created",
        "audit-clean",
        "update-sync-copy-warning",
    ];
    for &loc in Locale::ALL {
        let b = Bundle::new(loc).unwrap();
        for &k in &probe_keys {
            let out = b.t(
                k,
                &[
                    ("id", "x"),
                    ("count", "1"),
                    ("src", "src-bin"),
                    ("dst", "dst-bin"),
                    ("reason", "denied"),
                ],
            );
            assert_ne!(
                out, k,
                "locale {loc:?} returned the key verbatim for `{k}` — message is missing"
            );
        }
    }
}
