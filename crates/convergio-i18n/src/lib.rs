//! # convergio-i18n — P5 enforcement
//!
//! Fluent-backed message bundles for every user-facing string in
//! Convergio. Italian and English are first-class. New locales land
//! by adding a directory under `locales/<lang>/` with one or more
//! `.ftl` files.
//!
//! ## Locale resolution
//!
//! Callers normally use [`detect_locale`] to pick the right bundle:
//!
//! 1. The explicit `--lang` flag (passed as `Some("it")` etc.)
//! 2. `CONVERGIO_LANG` environment variable
//! 3. `LANG` / `LC_ALL` environment variable (first 2 chars: `it_IT.UTF-8` → `it`)
//! 4. Fallback `en`
//!
//! ## API
//!
//! ```
//! use convergio_i18n::{Bundle, Locale};
//!
//! let bundle = Bundle::new(Locale::It).expect("valid locale");
//! let msg = bundle.t("plan-created", &[("id", "abc-123")]);
//! assert_eq!(msg, "Piano creato: abc-123");
//! ```
//!
//! See [CONSTITUTION.md § P5](../../../CONSTITUTION.md) for the
//! product principle this crate implements.

#![forbid(unsafe_code)]

mod bundle;
mod error;
mod locale;

pub use bundle::Bundle;
pub use error::{I18nError, Result};
pub use locale::{detect_locale, Locale};
