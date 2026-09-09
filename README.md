# Spaceship CLI

A CLI tool wrapping the [Spaceship API](https://docs.spaceship.dev/), optimized for LLM agent consumption. Built with [Spectre.Console](https://spectreconsole.net/) and .NET 10.

## Install

```bash
dotnet tool install --global Spaceship.Console
```

Set your credentials:

```bash
export SPACESHIP_API_KEY="your-key-here"
export SPACESHIP_API_SECRET="your-secret-here"
```

Or pass them per-command with `--api-key` and `--api-secret`.

## Output

**YAML by default** — more token-efficient for LLMs (no braces, quotes, or commas). Switch with `--format json` or `--format table`.

## Commands

```
spaceship domains list [--take] [--skip] [--order-by]         List all domains
spaceship domains get <domain>                                 Domain details
spaceship domains check <domain>                               Check availability
spaceship domains check-batch <domains>                        Batch availability (comma-separated, max 20)
spaceship domains register <domain> --registrant <id> [opts]   Register a domain
spaceship domains delete <domain>                              Delete a domain
spaceship domains renew <domain> --expiration-date <date>      Renew a domain
spaceship domains restore <domain>                             Restore a deleted domain
spaceship domains autorenew <domain> --enable|--disable        Update auto-renewal
spaceship domains nameservers <domain> --provider <p> [opts]   Update nameservers
spaceship domains contacts <domain> --registrant <id> [opts]   Update domain contacts
spaceship domains privacy <domain> --level public|high         Update privacy preference
spaceship domains auth-code <domain>                           Get transfer auth code
spaceship domains transfer <domain> --registrant <id> [opts]   Request domain transfer
spaceship domains transfer-status <domain>                     Get transfer status
spaceship domains transfer-lock <domain> --lock|--unlock       Update transfer lock
spaceship dns list <domain> [--type] [--name]                  List DNS records
spaceship dns save <domain> [--file]                           Save DNS records (stdin or file)
spaceship dns delete <domain> [--file]                         Delete DNS records (stdin or file)
spaceship contacts save --first-name --last-name --email ...   Save contact details
spaceship contacts get <id>                                    Get contact details
spaceship sellerhub list [--take] [--skip]                     List SellerHub domains
spaceship sellerhub get <id>                                   Get SellerHub domain
spaceship sellerhub create [--file]                            Create SellerHub domain
spaceship sellerhub update <id> [--file]                       Update SellerHub domain
spaceship sellerhub delete <id>                                Delete SellerHub domain
spaceship sellerhub checkout [--file]                          Create checkout link
spaceship sellerhub verification <id>                          Get verification records
spaceship operations get <id>                                  Get async operation status
```

## Examples

```bash
# List your domains
spaceship domains list --take 10

# Check if a domain is available
spaceship domains check example.com

# Batch check multiple domains
spaceship domains check-batch "example.com,example.net,example.org"

# Get domain details
spaceship domains get example.com

# List DNS records
spaceship dns list example.com

# Save DNS records from a file
spaceship dns save example.com --file records.json

# DNS record format (records.json example):
# Note: Different record types use different fields:
# - A/AAAA records use "address"
# - CNAME records use "cname"
# - MX records use "exchange" and "preference"
# - TXT records use "value"
#
# "save" does not overwrite an existing record with the same name and type --
# it fails with 422 "<TYPE> with host <name> already exists". To change a
# record, delete it first, then save the new one.
[
  {
    "name": "@",
    "type": "A",
    "address": "192.0.2.1",
    "ttl": 300
  },
  {
    "name": "www",
    "type": "CNAME",
    "cname": "example.com",
    "ttl": 300
  },
  {
    "name": "_acme-challenge",
    "type": "CNAME",
    "cname": "verification.example.net",
    "ttl": 300
  }
]

# Delete DNS records (match on name + type + value; ttl is not required)
echo '[{"name":"www","type":"CNAME","cname":"example.com"}]' | spaceship dns delete example.com

# Create a contact
spaceship contacts save --first-name John --last-name Doe --email john@example.com \
  --address "123 Main St" --city "New York" --country US

# Check async operation status
spaceship operations get op_abc123

# JSON output for jq pipelines
spaceship domains list --format json | jq '.items[] | .name'
```

## Global Options

| Flag | Description |
|---|---|
| `--api-key <key>` | Override `SPACESHIP_API_KEY` env var |
| `--api-secret <secret>` | Override `SPACESHIP_API_SECRET` env var |
| `--format yaml\|json\|table` | Output format (default: yaml) |
| `--verbose` | Print HTTP method, URL, and status code to stderr |
| `--no-color` | Disable colored output |

## Error Handling

Errors are written to stderr as YAML with predictable exit codes:

- `0` — success
- `1` — user/input error (bad arguments, not found)
- `2` — API/network error (rate limit, connection failure)
