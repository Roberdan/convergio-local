# convergio-cli (`cvg`)

Pure HTTP client for the local Convergio daemon.

```bash
cargo run -p convergio-cli -- health
cargo run -p convergio-cli -- plan create "my plan"
cargo run -p convergio-cli -- plan list
cargo run -p convergio-cli -- solve "write docs"
cargo run -p convergio-cli -- dispatch
cargo run -p convergio-cli -- validate <plan_id>
```

The CLI does not import `convergio-server` or internal server crates.
All inputs and outputs go through HTTP.

## Configuration

| Variable / flag | Default | Notes |
|-----------------|---------|-------|
| `CONVERGIO_URL` / `--url` | `http://127.0.0.1:8420` | daemon base URL |
| `CONVERGIO_LANG` / `--lang` | detected from environment, fallback `en` | `en` and `it` ship today |
