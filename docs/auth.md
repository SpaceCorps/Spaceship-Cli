---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the Spaceship CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for Spaceship CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the Spaceship CLI.

## Overview
The Spaceship CLI interfaces directly with the Spaceship public REST API (`https://spaceship.dev/api/v1/`). Authentication requires two distinct credentials:
- **API Key** (sent via the `X-API-Key` HTTP header)
- **API Secret** (sent via the `X-API-Secret` HTTP header)

Credentials can be stored securely in the host operating system's native keychain, supplied directly via environment variables, or passed on standard input or command-line flags.

## Prerequisites
- A Spaceship account ([spaceship.com](https://spaceship.com))
- An API Key and API Secret generated from the Spaceship dashboard:
  Navigate to **Settings &rarr; API & Developers** (`https://www.spaceship.com/application/api-settings/`)
- Spaceship CLI installed on your machine (`cargo install --git https://github.com/SpaceCorps/Spaceship-Cli --locked`)

## Authentication Methods

### 1. Interactive Login (`spaceship login`)
The recommended flow for local developer machines:
```bash
# Log in and save credentials under the default account
spaceship login

# Or save credentials under a named account (e.g., 'work' or 'client-prod')
spaceship login work
```
1. The CLI prompts for your **Spaceship API Key** (input is visible).
2. The CLI prompts for your **Spaceship API Secret** (input characters are masked).
3. The CLI validates credentials immediately with a test request to `GET /v1/domains?take=1`.
4. Upon successful validation, both credentials are saved securely in your native OS keystore under the specified account alias (default: `default`).

### 2. Headless / CI/CD Login via Stdin
For headless CI/CD environments, Docker containers, or autonomous agent runners, supply the key and secret via standard input:
```bash
# Pass key and secret separated by newline or space
printf "%s\n%s\n" "$SPACESHIP_API_KEY" "$SPACESHIP_API_SECRET" | spaceship login ci --api-key-stdin
```

### 3. Non-Interactive Login via Flags
You can also pass credentials directly to `spaceship login`:
```bash
spaceship login prod --api-key "$SPACESHIP_API_KEY" --api-secret "$SPACESHIP_API_SECRET"
```

### 4. Direct Flags without Stored Accounts
Commands can be executed directly with explicit `--api-key` and `--api-secret` flags, bypassing local keystore resolution:
```bash
spaceship domains list --api-key "$SPACESHIP_API_KEY" --api-secret "$SPACESHIP_API_SECRET"
```

## Environment Variables
The CLI checks the environment for credentials when no account flag is specified or when operating in automated environments:
- `SPACESHIP_API_KEY`: API key used if not specified via account or flags.
- `SPACESHIP_API_SECRET`: API secret paired with `SPACESHIP_API_KEY`.
- `SPACESHIP_ACCOUNT`: Default account name to use for operations when `--account` (`-a`) is omitted.
- `SPACESHIP_ALLOW_PLAINTEXT_STORE`: Set to `1` or `true` to allow fallback to plaintext credential file storage on systems without a supported OS keychain service.

## Multi-Account Management
Switch, verify, and inspect accounts safely:
```bash
# List configured accounts
spaceship accounts list

# Check live API connectivity for all configured accounts
spaceship accounts list --check

# Test credentials for a specific account
spaceship accounts test work

# Remove an account from this machine
spaceship accounts remove work --yes
```

## Credential Resolution Precedence
When executing an API command, credentials are resolved in this order:
1. Explicit `--api-key` and `--api-secret` command-line flags.
2. Keystore credentials for the account specified via `--account <name>` (or `-a <name>`).
3. Keystore credentials for the account specified via `SPACESHIP_ACCOUNT`.
4. Environment variables `SPACESHIP_API_KEY` and `SPACESHIP_API_SECRET`.
5. Keystore credentials for the `default` account alias.

## Error Handling
When authentication fails, commands exit with non-zero exit codes and output standardized JSON error envelopes on stderr:
- `auth_required` (exit code 1): Missing credentials, invalid API key, or rejected API secret.
- `no_account` (exit code 2): Specified account does not exist in keystore or configuration.
- `not_found` (exit code 3): The requested domain, record, or resource does not exist.
- `rate_limited` (exit code 4): Spaceship API rate limits exceeded.
- `invalid_input` (exit code 5): Invalid argument, missing required parameter, or bad JSON input.
- `server_error` (exit code 6): Spaceship upstream API error (5xx).
- `unknown` (exit code 7): Network connection or I/O failure.

Example error envelope:
```json
{
  "error": "Spaceship API authentication failed (HTTP 401 Unauthorized)",
  "code": "auth_required",
  "detail": "Invalid API key or secret",
  "remediation": "Run 'spaceship login [account]' or check SPACESHIP_API_KEY and SPACESHIP_API_SECRET"
}
```

## Security Best Practices
1. **Never Commit API Secrets**: Keep `.env` or plaintext credential files out of version control.
2. **Rely on Native OS Keystores**: The CLI integrates directly with macOS Keychain, Windows DPAPI, and Linux Secret Service.
3. **Restrict API Keys**: If Spaceship allows IP restrictions or permission scoping, restrict your keys to the minimal necessary IP ranges and permissions.
4. **Machine Verification**: When writing agent automation scripts, always pass `--json` to reliably capture structured output and error codes.
