# convergio-cli (`cvg`)

Pure HTTP client for the Convergio daemon.

```bash
cargo run -p convergio-cli -- health
cargo run -p convergio-cli -- plan create "my plan"
cargo run -p convergio-cli -- plan list
cargo run -p convergio-cli -- audit verify
```

The CLI does **not** import `convergio-server` or any internal
crate — by design. All inputs and outputs go through HTTP. A contract
test in `tests/` enforces this.

## Configuration

| Variable | Default | Notes |
|----------|---------|-------|
| `CONVERGIO_URL` | `http://127.0.0.1:8420` | Daemon base URL |

Override per-call with `--url`.
