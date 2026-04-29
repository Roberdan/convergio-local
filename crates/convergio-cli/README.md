# convergio-cli (`cvg`)

Pure HTTP client for the local Convergio daemon.

```bash
cargo run -p convergio-cli -- health
cargo run -p convergio-cli -- setup
cargo run -p convergio-cli -- doctor
cargo run -p convergio-cli -- plan create "my plan"
cargo run -p convergio-cli -- plan list
cargo run -p convergio-cli -- task list <plan_id>
cargo run -p convergio-cli -- evidence add <task_id> --kind code --payload '{"diff":"fn main() {}"}'
cargo run -p convergio-cli -- solve "write docs"
cargo run -p convergio-cli -- dispatch
cargo run -p convergio-cli -- validate <plan_id>
cargo run -p convergio-cli -- demo
```

The CLI does not import `convergio-server` or internal server crates.
All inputs and outputs go through HTTP.

## Configuration

| Variable / flag | Default | Notes |
|-----------------|---------|-------|
| `CONVERGIO_URL` / `--url` | `http://127.0.0.1:8420` | daemon base URL |
| `CONVERGIO_LANG` / `--lang` | detected from environment, fallback `en` | `en` and `it` ship today |

`cvg setup` creates `~/.convergio/config.toml` and local adapter
directories. `cvg doctor` checks the local daemon, binaries, audit chain
and setup state; use `cvg doctor --json` from scripts or agents.
