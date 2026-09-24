//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a CLAUDE.md; `--json` gives the same rules as data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "spaceship",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, check credentials or scope",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call arguments",
                "7" => "no_account - run spaceship accounts list or pass credentials",
            },
        });
        return;
    }
    println!("{README}");
}

pub const API_VERSION: &str = "v1";

const RULES: &[&str] = &[
    "Pass --account <name> (or configure SPACESHIP_API_KEY and SPACESHIP_API_SECRET).",
    "Run 'spaceship accounts list' first if you do not know which accounts exist; ask the human which to use.",
    "On code auth_required, stop and surface the remediation string. Do not retry without fixing credentials or permissions.",
    "Async domain operations (register, renew, restore, transfer) return an asyncOperationId; poll it with 'spaceship operations get <id>'.",
    "DNS save adds records or updates matching TTLs. To change a record value, delete the old record first or pass --force.",
    "DNS delete matches on name, type, and value.",
    "SellerHub and SafePay transactions can be created/updated with JSON files or piped via stdin.",
    "Deletes are irreversible. Read the resource back before deleting it.",
    "Use --json when you are going to parse the output in code or jq.",
];

const README: &str = r#"# spaceship - agent operating manual

A native CLI over the Spaceship API (v1): domains, personal nameservers, DNS records, contacts,
SellerHub listings and SafePay escrow, Hyperlift applications, and async operations. Results are
YAML on stdout by default, errors are YAML on stderr, and `--json` switches both to JSON.

## Authentication and Accounts

Spaceship requires both an API Key (`X-API-Key`) and an API Secret (`X-API-Secret`).
Credentials can be provided in three ways:

1. **Named accounts in the OS keystore** (macOS Keychain, Windows DPAPI, Linux Secret Service):
       spaceship login [<name>] [--api-key <key>] [--api-secret <secret>]
       spaceship accounts add <name> --api-key <key> --api-secret <secret>
       spaceship domains list -a <name>

2. **Environment variables**:
       export SPACESHIP_API_KEY="your-api-key"
       export SPACESHIP_API_SECRET="your-api-secret"
       spaceship domains list

3. **Direct CLI flags**:
       spaceship domains list --api-key <key> --api-secret <secret>

Each Spaceship API key has specific scopes (`domains:read`, `dnsrecords:write`, `sellerhub:read`,
`hyperlift:execute`, etc.). If a scope is missing, the API returns `auth_required` (HTTP 403)
with "Insufficient permissions".

## Reading and Managing Domains

    spaceship domains list [--take <n>] [--skip <n>] [--order-by <field>]
    spaceship domains get <domain>
    spaceship domains check <domain>
    spaceship domains check-batch "example.com,example.net"
    spaceship domains register <domain> --registrant <id> [--years <n>] [--auto-renew]
    spaceship domains renew <domain> --expiration-date <date> [--years <n>]
    spaceship domains restore <domain>
    spaceship domains autorenew <domain> --enable|--disable
    spaceship domains nameservers <domain> --provider basic|custom [--hosts <h1,h2>]
    spaceship domains contacts <domain> --registrant <id> [--admin <id>] [--tech <id>] [--billing <id>]
    spaceship domains privacy <domain> --level public|high
    spaceship domains email-protection <domain> --enable|--disable
    spaceship domains auth-code <domain>
    spaceship domains transfer <domain> --registrant <id> [--auth-code <code>]
    spaceship domains transfer-status <domain>
    spaceship domains transfer-lock <domain> --lock|--unlock
    spaceship domains personal-ns list <domain>
    spaceship domains personal-ns save <domain> <host> --ips <ip1,ip2> [--rename <host>]
    spaceship domains personal-ns delete <domain> <host>

## DNS Records

    spaceship dns list <domain> [--type <type>] [--name <name>] [--all]
    spaceship dns save <domain> [--file <path>] [--force]
    spaceship dns delete <domain> [--file <path>]

`dns save` adds records or updates TTLs for exact matches. To replace a value, either pass
`--force` or delete the previous record first. Input is accepted via stdin or `--file`.

## Contacts

    spaceship contacts save --first-name <fn> --last-name <ln> --email <e> --address <addr> --city <city> --country <c> --phone <p>
    spaceship contacts get <id>
    spaceship contacts attributes save [--file <path>]
    spaceship contacts attributes get <id>

## SellerHub and SafePay

    spaceship sellerhub list [--take <n>] [--skip <n>]
    spaceship sellerhub get <domain>
    spaceship sellerhub create [--file <path>]
    spaceship sellerhub update <domain> [--file <path>]
    spaceship sellerhub delete <domain>
    spaceship sellerhub checkout [--file <path>]
    spaceship sellerhub verification
    spaceship sellerhub sold [--take <n>] [--cursor <c>] [--from <dt>] [--to <dt>]
    spaceship sellerhub safepay list
    spaceship sellerhub safepay get <id>
    spaceship sellerhub safepay create [--file <path>]

## Hyperlift Application Hosting

    spaceship hyperlift list
    spaceship hyperlift get <id>
    spaceship hyperlift build <id>
    spaceship hyperlift build-logs <id> [--take <n>] [--cursor <c>]
    spaceship hyperlift logs <id> [--take <n>] [--cursor <c>]
    spaceship hyperlift metrics <id> --start <dt> --end <dt> --interval <int> --metrics <m>
    spaceship hyperlift restart <id>
    spaceship hyperlift scale <id> --scale 0|1
    spaceship hyperlift env get <id>
    spaceship hyperlift env set <id> [--file <path>]

## Async Operations

Operations like domain registration, renewal, and transfer return an `asyncOperationId`.
Check status with:

    spaceship operations get <id>

## Exit Codes

    0  ok
    1  error          unclassified - report it and stop
    2  network        retry once, then stop
    3  auth_required  check API key, secret, and scopes
    4  not_found      the resource does not exist; do not retry
    5  rate_limited   back off before trying again
    6  invalid_input  fix the arguments
    7  no_account     run 'spaceship accounts list' or set credentials"#;
