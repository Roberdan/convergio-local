# AGENTS.md — convergio-i18n

For repo-wide rules see [../../AGENTS.md](../../AGENTS.md).

This crate protects the internationalization-first rule.

## Invariants

- English and Italian ship together for localized strings.
- Do not hardcode new user-facing CLI strings outside the i18n path when
  the surface is localized.
- Keep fallback behavior explicit and tested.
- Accessibility matters: translated strings must still be clear in plain
  terminal output.
