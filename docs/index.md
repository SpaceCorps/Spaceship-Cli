---
title: "Spaceship CLI"
description: "A blazing fast native command-line tool and agent interface for the Spaceship domain registration and web services platform API (v1). Built in Rust for developers and autonomous AI agents."
author: "SpaceCorps"
date: "2026-09-24"
canonical: "https://spacecorps.github.io/Spaceship-Cli/index.md"
---

# Spaceship CLI

A blazing fast native command-line tool and agent interface for the Spaceship domain registration and web services platform API (v1). Built in Rust Edition 2024 for developers and autonomous AI agents.

## Quickstart

```bash
# Authenticate interactively (prompts for API Key & Secret)
spaceship login

# Or non-interactively in headless CI/CD environments
printf "%s\n%s\n" "$SPACESHIP_API_KEY" "$SPACESHIP_API_SECRET" | spaceship login ci --api-key-stdin
```

## Features

- **Blazing Fast Native Rust**: Sub-millisecond startup times with zero external runtime dependencies.
- **AI Agent Native**: Structured JSON output (`--json`), standardized error envelopes on stderr, and built-in `spaceship agent-readme`.
- **Secure Keystore Integration**: Dual API Key & Secret storage in native macOS Keychain, Windows DPAPI, and Linux Secret Service.
- **Multi-Account Workspaces**: Isolate personal, client, and production Spaceship accounts safely with `-a <account>`.
- **Full API Parity**: Comprehensive coverage across domains, personal nameservers, DNS records, contacts, sellerhub marketplace, hyperlift deployments, and operations.

## When to Use This CLI

Use the `spaceship` CLI whenever you need to:
- Check domain availability, register new domains, renew registrations, or transfer domains into Spaceship.
- Inspect, create, edit, or delete DNS records (A, AAAA, CNAME, MX, TXT, NS, SRV, CAA, ALIAS) across your domains.
- Configure personal/custom nameservers with IPv4 and IPv6 glue records.
- Manage WHOIS contact details and enable or disable domain privacy protection.
- List domains on the SellerHub marketplace, configure SafePay verification, and update prices.
- Trigger Hyperlift application deployments and manage environment variables.
- Monitor asynchronous background operations (registration, transfer, restore) via operation IDs.
- Automate domain provisioning and DNS management within CI/CD pipelines or autonomous AI agent loops.

## Documentation Links

- [llms.txt](https://spacecorps.github.io/Spaceship-Cli/llms.txt)
- [Full Agent Manual](https://spacecorps.github.io/Spaceship-Cli/llms-full.txt)
- [Pricing Guide](https://spacecorps.github.io/Spaceship-Cli/pricing.md)
- [Authentication Guide](https://spacecorps.github.io/Spaceship-Cli/auth.md)
- [GitHub Repository](https://github.com/SpaceCorps/Spaceship-Cli)
