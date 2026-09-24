# AGENTS.md

Developer and autonomous agent notes for `Spaceship-Cli`.

`spaceship` is a native Rust CLI for the Spaceship v1 REST API, engineered for humans and autonomous AI agents. It replaced the NielsBosma .NET prototype `Spaceship.Console` while preserving 100% of its command interface, argument flags, API paths, and payloads, while adding SpaceCorps enterprise multi-account security and OS keystores (Keychain, DPAPI, secret-tool).

For the operating manual the *agent* reads, run `spaceship agent-readme` — that text lives in `src/readme.rs` and is the tool's machine interface for autonomous tool-use loops. This file documents the internal codebase architecture.

## Commands

```bash
cargo build --release              # target/release/spaceship
cargo test                         # unit tests + tests/cli.rs against mock API
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install to PATH
```

Use a throwaway directory when running tests or development scripts:

```bash
export SPACESHIP_CONFIG_DIR=$(mktemp -d) SPACESHIP_SECRET_STORE=plaintext SPACESHIP_ALLOW_PLAINTEXT_STORE=1
```

| Variable | Effect |
| --- | --- |
| `SPACESHIP_CONFIG_DIR` | Overrides the configuration directory path |
| `SPACESHIP_SECRET_STORE` | Forces a specific backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `SPACESHIP_ALLOW_PLAINTEXT_STORE=1` | Permits the plaintext JSON store fallback if no OS keystore exists |
| `SPACESHIP_API_URL` | Overrides the API base URL (`https://spaceship.dev/api/v1/`) — used by `tests/cli.rs` |
| `SPACESHIP_API_KEY` | Environment fallback for API key |
| `SPACESHIP_API_SECRET` | Environment fallback for API secret |

## Codebase Layout

```
src/
  main.rs          argument parsing, --json pre-scan, clap error envelopes
  cli.rs           clap derive hierarchy, command options, and flags
  commands/
    mod.rs         command dispatcher and read_json_input helper
    login.rs       spaceship login command implementation
    accounts.rs    spaceship accounts add|list|test|remove
    domains.rs     domains *, personal-ns *
    dns.rs         dns records list, save, delete (with filtering & paging)
    contacts.rs    contacts save, get, attributes
    sellerhub.rs   sellerhub *, safepay *
    hyperlift.rs   hyperlift *, build, logs, metrics, env
    operations.rs  async-operations get
  client.rs        blocking HTTP (ureq 3.4 + rustls), status -> ErrorCode mapping
  error.rs         ErrorCode enum, Error struct { code, message, detail, remediation }
  output.rs        YAML by default, JSON with --json, write_error, obj! macro
  account.rs       credential resolution (keystore, environment, direct flags)
  config.rs        config.yaml storage, atomic writes, owner-restricted permissions, locking
  secrets.rs       macOS Keychain, Linux secret-tool, Windows DPAPI, plaintext fallback
  readme.rs        built-in agent-readme manual and rules
tests/
  cli.rs           in-process TCP mock HTTP server test suite
```

## Architectural Tenets

1. **Blocking HTTP over Tokio**: CLIs perform sequential or scoped parallel network requests. Using `ureq` avoids the compilation time and startup latency of async runtimes.
2. **OS Keystore Integration**: Secrets never touch `config.yaml` in plaintext without explicit opt-in (`SPACESHIP_ALLOW_PLAINTEXT_STORE=1`).
3. **Structured Outputs**: YAML is output by default for terminal and LLM readability; `--json` provides structured JSON for pipelines.
4. **Async Operation Tracking**: Endpoints returning HTTP 202 (domain registrations, renewals, restores, transfers) attach `asyncOperationId` from response headers into the result object so agents can immediately poll `spaceship operations get <id>`.
5. **Deterministic Exit Codes**:
   - `0`: ok
   - `1`: error (general)
   - `2`: network / server error
   - `3`: auth_required (check credentials and scopes)
   - `4`: not_found
   - `5`: rate_limited
   - `6`: invalid_input
   - `7`: no_account
