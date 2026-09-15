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

Or pass them per-command with `--api-key` and `--api-secret`. Each endpoint needs its own API key scope
(`domains:read`, `dnsrecords:write`, `sellerhub:read`, `hyperlift:execute`, ...); a missing scope returns
`Insufficient Permissions.`

## Output

**YAML by default** — more token-efficient for LLMs (no braces, quotes, or commas). Switch with `--format json` or `--format table`.

## Commands

```
spaceship domains list [--take] [--skip] [--order-by]         List all domains
spaceship domains get <domain>                                 Domain details
spaceship domains check <domain>                               Check availability
spaceship domains check-batch <domains>                        Batch availability (comma-separated, max 20)
spaceship domains register <domain> --registrant <id> [opts]   Register a domain (async)
spaceship domains delete <domain>                              Delete a domain (API returns 501 for now)
spaceship domains renew <domain> --expiration-date <date>      Renew a domain (async)
spaceship domains restore <domain>                             Restore a deleted domain (async)
spaceship domains autorenew <domain> --enable|--disable        Update auto-renewal
spaceship domains nameservers <domain> --provider <p> [opts]   Update nameservers
spaceship domains contacts <domain> --registrant <id> [opts]   Update domain contacts
spaceship domains privacy <domain> --level public|high         Update privacy preference
spaceship domains email-protection <domain> --enable|--disable Update email protection (contact form)
spaceship domains auth-code <domain>                           Get transfer auth code
spaceship domains transfer <domain> --registrant <id> [opts]   Request domain transfer (async)
spaceship domains transfer-status <domain>                     Get transfer status
spaceship domains transfer-lock <domain> --lock|--unlock       Update transfer lock
spaceship domains personal-ns list <domain>                    List personal nameservers (glue records)
spaceship domains personal-ns save <domain> <host> --ips <ips> Create/update a personal nameserver [--rename]
spaceship domains personal-ns delete <domain> <host>           Delete a personal nameserver
spaceship dns list <domain> [--type] [--name] [--all]          List DNS records [--take 1-500] [--skip] [--order-by]
spaceship dns save <domain> [--file] [--force]                 Add records / update TTLs (stdin or file)
spaceship dns delete <domain> [--file]                         Delete DNS records (stdin or file)
spaceship contacts save --first-name --last-name --phone ...   Save contact details
spaceship contacts get <id>                                    Get contact details
spaceship contacts attributes save [--file]                    Save contact attributes (stdin or file)
spaceship contacts attributes get <id>                         Get contact attributes
spaceship sellerhub list [--take] [--skip]                     List SellerHub domains
spaceship sellerhub get <domain>                               Get SellerHub domain
spaceship sellerhub create [--file]                            Create SellerHub domain
spaceship sellerhub update <domain> [--file]                   Update SellerHub domain
spaceship sellerhub delete <domain>                            Delete SellerHub domain
spaceship sellerhub checkout [--file]                          Create checkout link
spaceship sellerhub verification                               Get ownership verification record options
spaceship sellerhub sold [--take] [--cursor] [--from] [--to]   List sold domains
spaceship sellerhub safepay list [--take] [--skip]             List SafePay transactions
spaceship sellerhub safepay get <transaction-id>               Get a SafePay transaction
spaceship sellerhub safepay create [--file]                    Create a SafePay transaction
spaceship hyperlift list [--take] [--skip]                     List Hyperlift applications
spaceship hyperlift get <id>                                   Get an application
spaceship hyperlift build <id>                                 Start a build
spaceship hyperlift build-logs <id> [--take] [--cursor]        Build logs
spaceship hyperlift logs <id> [--take] [--cursor]              Runtime logs
spaceship hyperlift metrics <id> --start --end --interval --metrics   Metrics
spaceship hyperlift restart <id>                               Restart an application
spaceship hyperlift scale <id> --scale 0|1                     Stop (0) or start (1) an application
spaceship hyperlift env get <id>                               Get environment variables
spaceship hyperlift env set <id> [--file]                      Replace ALL environment variables
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

# List DNS records (100 per page by default, up to 500)
spaceship dns list example.com
spaceship dns list example.com --all --order-by name

# Filter by type or name. The API has no filters, so these page through the whole zone.
spaceship dns list example.com --type CNAME
spaceship dns list example.com --name www

# Save DNS records from a file
spaceship dns save example.com --file records.json

# DNS record format (records.json example):
# Different record types use different fields:
# - A/AAAA: "address"
# - CNAME: "cname"
# - MX: "exchange" and "preference"
# - TXT: "value"
# - SRV: "service", "protocol", "priority", "weight", "port", "target"
# - CAA: "flag", "tag", "value"   NS: "nameserver"   ALIAS: "aliasName"   PTR: "pointer"
# - HTTPS/SVCB: "svcPriority", "targetName", "svcParams"
# - TLSA: "port", "protocol", "usage", "selector", "matching", "associationData"
#
# "save" adds records, or updates the TTL of a record that already matches exactly.
# It is not an upsert: saving a record with the same name and type but a different
# value either adds it next to the old one (a CNAME ends up with two targets) or is
# rejected with 422 by the API's conflict checks. To change a value, delete the old
# record first, then save the new one. --force turns the conflict checks off.
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

# Delete DNS records (match on name + type + value; ttl is not required; TXT values are case-sensitive)
echo '[{"name":"www","type":"CNAME","cname":"example.com"}]' | spaceship dns delete example.com

# Point ns1.example.com at your own server (glue record)
spaceship domains personal-ns save example.com ns1 --ips 192.0.2.10,2001:db8::10

# Create a contact
spaceship contacts save --first-name John --last-name Doe --email john@example.com \
  --address "123 Main St" --city "New York" --country US --phone +1.2125551234

# Register a domain, then poll the async operation it returns (asyncOperationId)
spaceship domains register example.com --registrant <contact-id> --years 1
spaceship operations get <asyncOperationId>

# Replace a Hyperlift application's environment (variables left out are deleted)
echo '{"NODE_ENV":"production","APPLICATION_PORT":"8080"}' | spaceship hyperlift env set <app-id>

# JSON output for jq pipelines
spaceship domains list --format json | jq '.items[] | .name'
```

## Global Options

| Flag | Description |
|---|---|
| `--api-key <key>` | Override `SPACESHIP_API_KEY` env var |
| `--api-secret <secret>` | Override `SPACESHIP_API_SECRET` env var |
| `--format yaml\|json\|table` | Output format (default: yaml) |
| `--verbose` | Print HTTP method, URL, status code and Spaceship operation ID to stderr |
| `--no-color` | Disable colored output |

## Error Handling

Errors are written to stderr as YAML with predictable exit codes:

- `0` — success
- `1` — user/input error (bad arguments, not found, insufficient permissions, validation)
- `2` — API/network error (rate limit, server error, connection failure)

Validation errors include the API's per-field details, e.g.
`Request validation error. - CNAME with host www already exists`.
