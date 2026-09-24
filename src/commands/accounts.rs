//! Accounts command implementations.

use std::io::{IsTerminal, Read};

use serde_json::Value;

use crate::client::Client;
use crate::config::{self, AccountConfig};
use crate::error::{Error, Result};
use crate::output;
use crate::secrets::{self, SecretEntry};
use crate::{cli::AccountsCommand, obj};

pub fn run(cmd: AccountsCommand) -> Result<()> {
    match cmd {
        AccountsCommand::Add { name, api_key, api_key_stdin, api_secret, api_secret_stdin, force, no_verify } => {
            add(name, api_key, api_key_stdin, api_secret, api_secret_stdin, force, no_verify)
        }
        AccountsCommand::List { check } => list(check),
        AccountsCommand::Test { name } => test(name),
        AccountsCommand::Remove { name, yes } => remove(name, yes),
    }
}

fn add(
    name: String,
    api_key: Option<String>,
    api_key_stdin: bool,
    api_secret: Option<String>,
    api_secret_stdin: bool,
    force: bool,
    no_verify: bool,
) -> Result<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(Error::invalid("Account name cannot be empty."));
    }

    let _lock = config::lock()?;
    let mut conf = config::load()?;

    if conf.find(&name).is_some() && !force {
        return Err(Error::invalid(format!("Account '{name}' already exists."))
            .fix(format!("spaceship accounts add {name} --force")));
    }

    let api_key = match (api_key, api_key_stdin) {
        (Some(k), false) => k.trim().to_string(),
        (None, true) => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf.trim().to_string()
        }
        (None, false) => {
            if !std::io::stdin().is_terminal() {
                return Err(Error::invalid("No API key provided. Pass --api-key or --api-key-stdin."));
            }
            eprint!("Enter Spaceship API Key: ");
            rpassword::read_password().map_err(|e| Error::other(e.to_string()))?.trim().to_string()
        }
        (Some(_), true) => return Err(Error::invalid("Cannot specify both --api-key and --api-key-stdin.")),
    };

    if api_key.is_empty() {
        return Err(Error::invalid("API key cannot be empty."));
    }

    let api_secret = match (api_secret, api_secret_stdin) {
        (Some(s), false) => s.trim().to_string(),
        (None, true) => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf.trim().to_string()
        }
        (None, false) => {
            if !std::io::stdin().is_terminal() {
                return Err(Error::invalid("No API secret provided. Pass --api-secret or --api-secret-stdin."));
            }
            eprint!("Enter Spaceship API Secret: ");
            rpassword::read_password().map_err(|e| Error::other(e.to_string()))?.trim().to_string()
        }
        (Some(_), true) => return Err(Error::invalid("Cannot specify both --api-secret and --api-secret-stdin.")),
    };

    if api_secret.is_empty() {
        return Err(Error::invalid("API secret cannot be empty."));
    }

    let store = secrets::store()?;

    if !no_verify {
        let client = Client::new(&api_key, &api_secret, false);
        let _ = client.get("domains?take=1")?;
    }

    store.set(&secrets::account_key(&name), &SecretEntry { api_key: api_key.clone(), api_secret })?;

    let identity = if api_key.len() > 8 { format!("{}...", &api_key[..8]) } else { api_key };

    conf.accounts.insert(name.clone(), AccountConfig { identity: identity.clone(), added_at: config::now_utc() });
    config::save(&conf)?;

    output::write(&obj! {
        "status" => "added",
        "name" => name,
        "identity" => identity,
        "secretStore" => store.name(),
    });

    Ok(())
}

fn list(check: bool) -> Result<()> {
    let conf = config::load()?;
    let store = secrets::store()?;

    let mut accounts = Vec::new();
    for (name, acct) in conf.sorted() {
        let key_status = if check {
            match store.get(&secrets::account_key(name))? {
                Some(creds) => {
                    let client = Client::new(&creds.api_key, &creds.api_secret, false);
                    match client.get("domains?take=1") {
                        Ok(_) => "valid",
                        Err(e) if e.code == crate::error::ErrorCode::AuthRequired => "rejected",
                        Err(_) => "unreachable",
                    }
                }
                None => "missing",
            }
        } else {
            "stored"
        };

        accounts.push(obj! {
            "name" => name,
            "identity" => acct.identity.clone(),
            "addedAt" => acct.added_at.clone(),
            "keyStatus" => key_status,
        });
    }

    output::write(&obj! {
        "count" => accounts.len(),
        "accounts" => Value::Array(accounts),
        "secretStore" => store.name(),
    });

    Ok(())
}

fn test(name: String) -> Result<()> {
    let conf = config::load()?;
    let Some((stored_name, _)) = conf.find(&name) else {
        return Err(Error::invalid(format!("No account named '{name}'.")).fix("spaceship accounts list"));
    };

    let store = secrets::store()?;
    let creds = store.get(&secrets::account_key(stored_name))?;
    let Some(creds) = creds else {
        return Err(Error::new(
            crate::error::ErrorCode::AuthRequired,
            format!("Account '{stored_name}' has no stored credentials."),
        )
        .fix(format!("spaceship accounts add {stored_name} --force")));
    };

    let client = Client::new(&creds.api_key, &creds.api_secret, false);
    client.get("domains?take=1")?;

    output::write(&obj! {
        "account" => stored_name,
        "keyStatus" => "valid",
    });

    Ok(())
}

fn remove(name: String, yes: bool) -> Result<()> {
    let _lock = config::lock()?;
    let mut conf = config::load()?;

    let Some((stored_name, _)) = conf.find(&name).map(|(k, v)| (k.clone(), v.clone())) else {
        return Err(Error::invalid(format!("No account named '{name}'.")).fix("spaceship accounts list"));
    };

    if !yes {
        if !std::io::stdin().is_terminal() {
            return Err(Error::invalid(format!("To remove account '{stored_name}' non-interactively, pass --yes."))
                .fix(format!("spaceship accounts remove {stored_name} --yes")));
        }
        eprint!("Remove account '{stored_name}' and delete stored credentials? [y/N]: ");
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }
    }

    let store = secrets::store()?;
    store.delete(&secrets::account_key(&stored_name))?;

    conf.accounts.shift_remove(&stored_name);
    config::save(&conf)?;

    output::write(&obj! {
        "status" => "removed",
        "name" => stored_name,
    });

    Ok(())
}
