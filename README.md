# Spaceship CLI

[![CI](https://github.com/SpaceCorps/Spaceship-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Spaceship-Cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/SpaceCorps/Spaceship-Cli?style=flat-square)](https://github.com/SpaceCorps/Spaceship-Cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-informational?style=flat-square)](https://spacecorps.github.io/Spaceship-Cli/)

A blazing fast, native Rust CLI for the [Spaceship API](https://docs.spaceship.dev/), optimized for developers and autonomous AI agents. Originally created as a .NET prototype by Niels Bosma, now fully ported to native Rust Edition 2024 by SpaceCorps.

## Installation

### Precompiled Binaries
Download the latest static binary for your architecture from [Releases](https://github.com/SpaceCorps/Spaceship-Cli/releases):
- **macOS** (Apple Silicon `aarch64` & Intel `x86_64`)
- **Linux** (musl static `x86_64` & `aarch64`)
- **Windows** (`x86_64` msvc)

### Install via Cargo
```bash
cargo install --git https://github.com/SpaceCorps/Spaceship-Cli --locked
```

## Authentication

Spaceship API requires both an **API Key** (`X-API-Key`) and an **API Secret** (`X-API-Secret`). You can configure them in three ways:

### 1. OS Keystore (Recommended)
Store your credentials securely in your operating system's native vault (macOS Keychain, Windows DPAPI, Linux Secret Service):
```bash
spaceship login
# Or add named accounts:
spaceship accounts add work --api-key "your-key" --api-secret "your-secret"
```

Use `-a <account>` or `--account <account>` to execute commands against a specific account:
```bash
spaceship domains list -a work
```

### 2. Environment Variables
```bash
export SPACESHIP_API_KEY="your-key-here"
export SPACESHIP_API_SECRET="your-secret-here"
```

### 3. Command Flags
Pass `--api-key` and `--api-secret` per command:
```bash
spaceship domains list --api-key "key" --api-secret "secret"
```

> **Note on Scopes:** Each Spaceship endpoint requires its own API key scope (e.g. `domains:read`, `dnsrecords:write`, `sellerhub:read`, `hyperlift:execute`). A missing scope returns HTTP 403 `auth_required` with `Insufficient permissions`.

## Output Formats

- **YAML (default)**: Clean, token-efficient, human-readable terminal output without JSON quotation noise.
- **JSON (`--json` or `--format json`)**: Structured JSON for `jq` pipelines and programmatic LLM agent tool loops.

## Commands Overview

| Command | Description |
|---|---|
| `spaceship domains list [--take] [--skip] [--order-by]` | List all registered domains |
| `spaceship domains get <domain>` | Domain details |
| `spaceship domains check <domain>` | Check domain availability |
| `spaceship domains check-batch <domains>` | Batch check availability (comma-separated, max 20) |
| `spaceship domains register <domain> --registrant <id>` | Register a domain (async) |
| `spaceship domains delete <domain>` | Delete a domain |
| `spaceship domains renew <domain> --expiration-date <date>` | Renew a domain (async) |
| `spaceship domains restore <domain>` | Restore a deleted domain (async) |
| `spaceship domains autorenew <domain> --enable\|--disable` | Update auto-renewal |
| `spaceship domains nameservers <domain> --provider <p>` | Update nameservers (basic or custom) |
| `spaceship domains contacts <domain> --registrant <id>` | Update domain contacts |
| `spaceship domains privacy <domain> --level public\|high` | Update privacy preference |
| `spaceship domains email-protection <domain> --enable\|--disable` | Update email protection |
| `spaceship domains auth-code <domain>` | Get transfer authorization code |
| `spaceship domains transfer <domain> --registrant <id>` | Request domain transfer (async) |
| `spaceship domains transfer-status <domain>` | Get transfer status |
| `spaceship domains transfer-lock <domain> --lock\|--unlock` | Update transfer lock |
| `spaceship domains personal-ns list <domain>` | List personal nameservers (glue records) |
| `spaceship domains personal-ns save <domain> <host> --ips <ips>` | Create or update personal nameserver |
| `spaceship domains personal-ns delete <domain> <host>` | Delete a personal nameserver |
| `spaceship dns list <domain> [--type] [--name] [--all]` | List DNS records (with local filtering) |
| `spaceship dns save <domain> [--file] [--force]` | Add records / update TTLs (stdin or file) |
| `spaceship dns delete <domain> [--file]` | Delete DNS records (stdin or file) |
| `spaceship contacts save --first-name --last-name ...` | Save contact details |
| `spaceship contacts get <id>` | Get contact details |
| `spaceship contacts attributes save [--file]` | Save registry-specific attributes |
| `spaceship contacts attributes get <id>` | Get contact attributes |
| `spaceship sellerhub list [--take] [--skip]` | List SellerHub domains |
| `spaceship sellerhub get <domain>` | Get SellerHub domain details |
| `spaceship sellerhub create [--file]` | Create SellerHub domain |
| `spaceship sellerhub update <domain> [--file]` | Update SellerHub domain |
| `spaceship sellerhub delete <domain>` | Delete SellerHub domain |
| `spaceship sellerhub checkout [--file]` | Create SellerHub checkout link |
| `spaceship sellerhub verification` | Get ownership verification records |
| `spaceship sellerhub sold [--take] [--cursor] [--from] [--to]` | List sold domains |
| `spaceship sellerhub safepay list [--take] [--skip]` | List SafePay transactions |
| `spaceship sellerhub safepay get <transaction-id>` | Get SafePay transaction |
| `spaceship sellerhub safepay create [--file]` | Create SafePay transaction |
| `spaceship hyperlift list [--take] [--skip]` | List Hyperlift applications |
| `spaceship hyperlift get <id>` | Get Hyperlift application |
| `spaceship hyperlift build <id>` | Trigger build |
| `spaceship hyperlift build-logs <id> [--take] [--cursor]` | Get build logs |
| `spaceship hyperlift logs <id> [--take] [--cursor]` | Get runtime logs |
| `spaceship hyperlift metrics <id> --start --end ...` | Query metrics |
| `spaceship hyperlift restart <id>` | Restart application |
| `spaceship hyperlift scale <id> --scale 0\|1` | Start (1) or stop (0) application |
| `spaceship hyperlift env get <id>` | Get environment variables |
| `spaceship hyperlift env set <id> [--file]` | Replace environment variables |
| `spaceship operations get <id>` | Get async operation status |
| `spaceship agent-readme [--json]` | Embedded operating manual for LLM agents |

## Examples

### Domain Management
```bash
# List your domains
spaceship domains list --take 10

# Check availability
spaceship domains check example.com

# Batch availability check
spaceship domains check-batch "example.com,example.net,example.org"

# Register a domain and poll the returned operation
spaceship domains register example.com --registrant <contact-id> --years 1
spaceship operations get <asyncOperationId>
```

### DNS Records
```bash
# List all records
spaceship dns list example.com

# Filter by type or record name (pages through the entire zone)
spaceship dns list example.com --type CNAME
spaceship dns list example.com --name www

# Save records from JSON file
spaceship dns save example.com --file records.json

# Delete records
echo '[{"name":"www","type":"CNAME","cname":"example.com"}]' | spaceship dns delete example.com
```

### JSON Pipelines
```bash
spaceship domains list --json | jq '.items[] | .name'
```

## Global Options

| Flag | Description |
|---|---|
| `-a, --account <name>` | Account name in keystore (`spaceship accounts list`) |
| `--api-key <key>` | Override `SPACESHIP_API_KEY` env var |
| `--api-secret <secret>` | Override `SPACESHIP_API_SECRET` env var |
| `--json` | Output raw JSON instead of YAML |
| `--format yaml\|json` | Alternative format selector |
| `--verbose` | Print HTTP method, URL, and status code to stderr |

## Error Envelopes & Exit Codes

On failure, structured error envelopes are printed to `stderr`:
```yaml
error: The resource does not exist.
code: not_found
detail: "HTTP 404: {\"message\": \"Domain not found\"}"
```

Exit codes:
- `0`: success
- `1`: general error
- `2`: network / server error (retry once)
- `3`: `auth_required` (check API key, secret, and scopes)
- `4`: `not_found`
- `5`: `rate_limited`
- `6`: `invalid_input`
- `7`: `no_account` (run `spaceship accounts list` or pass credentials)

## License

[MIT License](LICENSE) — Copyright (c) 2026 Niels Bosma, SpaceCorps contributors.
