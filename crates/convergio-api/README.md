# convergio-api

Shared agent-facing contract for Convergio integrations.

This crate owns the stable action names, schema version and structured
request/response shapes used by `convergio-mcp` and future adapters. It
does not call the daemon and it contains no business logic.
