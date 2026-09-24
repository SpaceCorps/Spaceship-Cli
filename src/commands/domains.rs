//! Domain management commands.

use serde_json::{Value, json};

use crate::cli::DomainsCommand;
use crate::client::{Client, query, seg};
use crate::error::{Error, Result};
use crate::output;

pub fn run(client: &Client, cmd: DomainsCommand) -> Result<()> {
    match cmd {
        DomainsCommand::List { take, skip, order_by } => list(client, take, skip, order_by),
        DomainsCommand::Get { domain } => get(client, &domain),
        DomainsCommand::Check { domain } => check(client, &domain),
        DomainsCommand::CheckBatch { domains } => check_batch(client, &domains),
        DomainsCommand::Register { domain, years, auto_renew, privacy, registrant, admin, tech, billing } => register(
            client,
            &domain,
            years,
            auto_renew,
            &privacy,
            &registrant,
            admin.as_deref(),
            tech.as_deref(),
            billing.as_deref(),
        ),
        DomainsCommand::Delete { domain } => delete(client, &domain),
        DomainsCommand::Renew { domain, years, expiration_date } => renew(client, &domain, years, &expiration_date),
        DomainsCommand::Restore { domain } => restore(client, &domain),
        DomainsCommand::Autorenew { domain, enable, disable } => autorenew(client, &domain, enable, disable),
        DomainsCommand::Nameservers { domain, provider, hosts } => {
            nameservers(client, &domain, &provider, hosts.as_deref())
        }
        DomainsCommand::Contacts { domain, registrant, admin, tech, billing } => {
            contacts(client, &domain, &registrant, admin.as_deref(), tech.as_deref(), billing.as_deref())
        }
        DomainsCommand::Privacy { domain, level } => privacy(client, &domain, &level),
        DomainsCommand::EmailProtection { domain, enable, disable } => {
            email_protection(client, &domain, enable, disable)
        }
        DomainsCommand::AuthCode { domain } => auth_code(client, &domain),
        DomainsCommand::Transfer { domain, auto_renew, privacy, registrant, admin, tech, billing, auth_code } => {
            transfer(
                client,
                &domain,
                auto_renew,
                &privacy,
                &registrant,
                admin.as_deref(),
                tech.as_deref(),
                billing.as_deref(),
                auth_code.as_deref(),
            )
        }
        DomainsCommand::TransferStatus { domain } => transfer_status(client, &domain),
        DomainsCommand::TransferLock { domain, lock, unlock } => transfer_lock(client, &domain, lock, unlock),
        DomainsCommand::PersonalNs(sub) => personal_ns(client, sub),
    }
}

fn list(client: &Client, take: u32, skip: u32, order_by: Option<String>) -> Result<()> {
    let q = query(&[("take", Some(take.to_string())), ("skip", Some(skip.to_string())), ("orderBy", order_by)]);
    let path = format!("domains{q}");
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn get(client: &Client, domain: &str) -> Result<()> {
    let path = format!("domains/{}", seg(domain));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn check(client: &Client, domain: &str) -> Result<()> {
    let path = format!("domains/{}/available", seg(domain));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn check_batch(client: &Client, domains_str: &str) -> Result<()> {
    let domains: Vec<String> =
        domains_str.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();

    if domains.len() > 20 {
        return Err(Error::invalid("Maximum 20 domains per batch check."));
    }

    let payload = json!({ "domains": domains });
    let res = client.post("domains/available", Some(&payload))?;
    output::write(&res);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn register(
    client: &Client,
    domain: &str,
    years: u32,
    auto_renew: bool,
    privacy: &str,
    registrant: &str,
    admin: Option<&str>,
    tech: Option<&str>,
    billing: Option<&str>,
) -> Result<()> {
    let mut contacts_map = serde_json::Map::new();
    contacts_map.insert("registrant".into(), Value::String(registrant.into()));
    if let Some(a) = admin {
        contacts_map.insert("admin".into(), Value::String(a.into()));
    }
    if let Some(t) = tech {
        contacts_map.insert("tech".into(), Value::String(t.into()));
    }
    if let Some(b) = billing {
        contacts_map.insert("billing".into(), Value::String(b.into()));
    }

    let body = json!({
        "autoRenew": auto_renew,
        "years": years,
        "privacyProtection": {
            "level": privacy,
            "userConsent": true
        },
        "contacts": Value::Object(contacts_map)
    });

    let path = format!("domains/{}", seg(domain));
    let res = client.post(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

fn delete(client: &Client, domain: &str) -> Result<()> {
    let path = format!("domains/{}", seg(domain));
    let res = client.delete(&path, None)?;
    output::write(&res);
    Ok(())
}

fn renew(client: &Client, domain: &str, years: u32, expiration_date: &str) -> Result<()> {
    let body = json!({
        "years": years,
        "currentExpirationDate": expiration_date
    });
    let path = format!("domains/{}/renew", seg(domain));
    let res = client.post(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

fn restore(client: &Client, domain: &str) -> Result<()> {
    let path = format!("domains/{}/restore", seg(domain));
    let res = client.post(&path, None)?;
    output::write(&res);
    Ok(())
}

fn autorenew(client: &Client, domain: &str, enable: bool, disable: bool) -> Result<()> {
    if !enable && !disable {
        return Err(Error::invalid("Specify --enable or --disable."));
    }
    if enable && disable {
        return Err(Error::invalid("Cannot specify both --enable and --disable."));
    }

    let body = json!({ "isEnabled": enable });
    let path = format!("domains/{}/autorenew", seg(domain));
    let res = client.put(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

fn nameservers(client: &Client, domain: &str, provider: &str, hosts: Option<&str>) -> Result<()> {
    let mut map = serde_json::Map::new();
    map.insert("provider".into(), Value::String(provider.into()));

    if provider.eq_ignore_ascii_case("custom") {
        let Some(hosts_str) = hosts.filter(|h| !h.trim().is_empty()) else {
            return Err(Error::invalid("--hosts is required when provider is custom."));
        };
        let parsed_hosts: Vec<String> =
            hosts_str.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
        if parsed_hosts.len() < 2 || parsed_hosts.len() > 12 {
            return Err(Error::invalid("Custom nameservers require 2-12 hosts."));
        }
        map.insert("hosts".into(), json!(parsed_hosts));
    }

    let body = Value::Object(map);
    let path = format!("domains/{}/nameservers", seg(domain));
    let res = client.put(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

fn contacts(
    client: &Client,
    domain: &str,
    registrant: &str,
    admin: Option<&str>,
    tech: Option<&str>,
    billing: Option<&str>,
) -> Result<()> {
    let mut contacts_map = serde_json::Map::new();
    contacts_map.insert("registrant".into(), Value::String(registrant.into()));
    if let Some(a) = admin {
        contacts_map.insert("admin".into(), Value::String(a.into()));
    }
    if let Some(t) = tech {
        contacts_map.insert("tech".into(), Value::String(t.into()));
    }
    if let Some(b) = billing {
        contacts_map.insert("billing".into(), Value::String(b.into()));
    }

    let path = format!("domains/{}/contacts", seg(domain));
    let res = client.put(&path, Some(&Value::Object(contacts_map)))?;
    output::write(&res);
    Ok(())
}

fn privacy(client: &Client, domain: &str, level: &str) -> Result<()> {
    if !level.eq_ignore_ascii_case("public") && !level.eq_ignore_ascii_case("high") {
        return Err(Error::invalid("Invalid privacy level. Must be 'public' or 'high'."));
    }

    let body = json!({
        "privacyLevel": level,
        "userConsent": true
    });
    let path = format!("domains/{}/privacy/preference", seg(domain));
    let res = client.put(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

fn email_protection(client: &Client, domain: &str, enable: bool, disable: bool) -> Result<()> {
    if !enable && !disable {
        return Err(Error::invalid("Specify --enable or --disable."));
    }
    if enable && disable {
        return Err(Error::invalid("Cannot specify both --enable and --disable."));
    }

    let body = json!({ "contactForm": enable });
    let path = format!("domains/{}/privacy/email-protection-preference", seg(domain));
    let res = client.put(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

fn auth_code(client: &Client, domain: &str) -> Result<()> {
    let path = format!("domains/{}/transfer/auth-code", seg(domain));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn transfer(
    client: &Client,
    domain: &str,
    auto_renew: bool,
    privacy: &str,
    registrant: &str,
    admin: Option<&str>,
    tech: Option<&str>,
    billing: Option<&str>,
    auth_code: Option<&str>,
) -> Result<()> {
    let mut contacts_map = serde_json::Map::new();
    contacts_map.insert("registrant".into(), Value::String(registrant.into()));
    if let Some(a) = admin {
        contacts_map.insert("admin".into(), Value::String(a.into()));
    }
    if let Some(t) = tech {
        contacts_map.insert("tech".into(), Value::String(t.into()));
    }
    if let Some(b) = billing {
        contacts_map.insert("billing".into(), Value::String(b.into()));
    }

    let mut body_map = serde_json::Map::new();
    body_map.insert("autoRenew".into(), Value::Bool(auto_renew));
    body_map.insert(
        "privacyProtection".into(),
        json!({
            "level": privacy,
            "userConsent": true
        }),
    );
    body_map.insert("contacts".into(), Value::Object(contacts_map));
    if let Some(code) = auth_code.filter(|c| !c.trim().is_empty()) {
        body_map.insert("authCode".into(), Value::String(code.into()));
    }

    let path = format!("domains/{}/transfer", seg(domain));
    let res = client.post(&path, Some(&Value::Object(body_map)))?;
    output::write(&res);
    Ok(())
}

fn transfer_status(client: &Client, domain: &str) -> Result<()> {
    let path = format!("domains/{}/transfer", seg(domain));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn transfer_lock(client: &Client, domain: &str, lock: bool, unlock: bool) -> Result<()> {
    if !lock && !unlock {
        return Err(Error::invalid("Specify --lock or --unlock."));
    }
    if lock && unlock {
        return Err(Error::invalid("Cannot specify both --lock and --unlock."));
    }

    let body = json!({ "isLocked": lock });
    let path = format!("domains/{}/transfer/lock", seg(domain));
    let res = client.put(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// personal-ns

use crate::cli::PersonalNsCommand;

fn normalize_host(host: &str, domain: &str) -> String {
    let label = host.trim().trim_end_matches('.');
    let suffix = format!(".{}", domain.trim().trim_end_matches('.'));
    if label.to_lowercase().ends_with(&suffix.to_lowercase()) {
        label[..label.len() - suffix.len()].to_string()
    } else {
        label.to_string()
    }
}

fn personal_ns(client: &Client, cmd: PersonalNsCommand) -> Result<()> {
    match cmd {
        PersonalNsCommand::List { domain } => {
            let path = format!("domains/{}/personal-nameservers", seg(&domain));
            let res = client.get(&path)?;
            output::write(&res);
            Ok(())
        }
        PersonalNsCommand::Save { domain, host, ips, rename } => {
            let ips_list: Vec<String> =
                ips.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
            if ips_list.is_empty() || ips_list.len() > 16 {
                return Err(Error::invalid("--ips needs 1-16 comma-separated addresses."));
            }

            let current = normalize_host(&host, &domain);
            let target_host = match rename.filter(|r| !r.trim().is_empty()) {
                Some(r) => normalize_host(&r, &domain),
                None => current.clone(),
            };

            let body = json!({
                "host": target_host,
                "ips": ips_list
            });

            let path = format!("domains/{}/personal-nameservers/{}", seg(&domain), seg(&current));
            let res = client.put(&path, Some(&body))?;
            output::write(&res);
            Ok(())
        }
        PersonalNsCommand::Delete { domain, host } => {
            let current = normalize_host(&host, &domain);
            let path = format!("domains/{}/personal-nameservers/{}", seg(&domain), seg(&current));
            let res = client.delete(&path, None)?;
            output::write(&res);
            Ok(())
        }
    }
}
