---
id: 0005
status: accepted
date: 2026-04-27
topics: [foundation, principles, i18n, accessibility]
related_adrs: []
touches_crates: []
last_validated: 2026-04-30
---

# 0005. Internationalization first (P5) — Italian + English from day one

- Status: accepted
- Date: 2026-04-27
- Deciders: Roberto, Claude (sessione 6)
- Tags: foundation, principles, i18n, accessibility

## Context and Problem Statement

Sessione 5 enshrined three sacred product principles. Roberto then
asked two follow-up questions:

1. "Have you put accessibility in the rules?" — Yes, P3.
2. "Have you put **multilingua by default** in the rules?" — Not
   explicitly. Two interpretations:
   - **A) Multi-programming-language rules**: P1 NoDebtGate covers 7
     languages, P4 NoStubGate covers all common comment families.
     This is *already* in place.
   - **B) Multi-natural-language UI**: nothing. Every CLI string was
     hardcoded English.

Roberto wanted **both**. Interpretation B becomes a new sacred
principle so that no future contributor can deprioritize it.

## Decision Drivers

- The product is built by an Italian founder for healthcare/regulated
  AI customers, several of whom will not read English fluently.
- Accessibility (P3) and internationalization are siblings: both
  remove unnecessary barriers between the user and the software.
- Adding i18n later is famously expensive. Doing it from day one is
  cheap if every new string is funneled through a bundle from the
  start.

## Considered Options

1. **Hardcode English, plan i18n for later**. Standard "we'll do it
   when there's a customer". Loses the cheap window.
2. **Roll our own message resolver**. Reinvented for no benefit.
3. **Fluent (Mozilla)** + a `convergio-i18n` crate with bundles per
   locale, locale resolution from CLI/env, default Italian + English.

## Decision Outcome

Chosen: **3 — Fluent + a dedicated i18n crate, P5 enshrined.**

### Rules

- Every user-facing string flows through `Bundle::t(key, args)`.
- No string concatenation for user-facing messages — Fluent's
  `{ $variable }` placeholders only.
- The machine-readable `code` of an API error stays English (it is
  contract). Only the human `message` is localized.
- Italian and English are first-class. Both ship in the binary via
  `include_str!`. Other locales contribute via PR adding
  `locales/<lang>/main.ftl` plus a `Locale` variant.
- Coverage gate: `cargo test -p convergio-i18n --test coverage`
  asserts every English key has an Italian translation and vice
  versa. A locale that ships partial keys does not ship.

### Locale resolution order

1. `--lang <tag>` CLI flag (passed to the binary)
2. `CONVERGIO_LANG` environment variable
3. `LANG` / `LC_ALL` environment variable (only the first 2 chars)
4. Fallback: `en`

### Architecture

```
crates/convergio-i18n/
├── locales/
│   ├── en/main.ftl
│   └── it/main.ftl
├── src/
│   ├── lib.rs
│   ├── bundle.rs   # FluentBundle wrapper, t() / t_n()
│   ├── locale.rs   # Locale enum + detect_locale()
│   └── error.rs
└── tests/
    └── coverage.rs # cross-locale coverage gate
```

### Positive consequences

- Italian users have a first-class experience day one.
- Adding a third locale (es, fr, de, ...) is a self-contained PR.
- Coverage gate prevents partial-locale ship.
- Reinforces P3 (accessibility): people who don't read English
  gracefully are not effectively excluded.

### Negative consequences

- Every user-facing string costs an extra `bundle.t(key, args)`
  call. Trivial overhead at the CLI layer; we will revisit if it
  becomes hot in HTTP responses.
- Fluent has its own syntax to learn. Worth it for plural-aware
  selectors and placeholders.
- Locale resolution happens in the CLI; HTTP `Accept-Language`
  support is a later milestone.

## Links

- Code: `crates/convergio-i18n/`
- Tests: `crates/convergio-i18n/tests/coverage.rs`,
  `crates/convergio-cli/tests/cli_smoke.rs` (verifies `--lang it`
  produces Italian output)
- Constitution: [CONSTITUTION.md § P5](../../CONSTITUTION.md)
- Related: ADR-0004 (sacred principles), P3 (accessibility)
