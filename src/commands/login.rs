//! Login command implementation.

use std::io::{IsTerminal, Read};

use crate::client::Client;
use crate::config::{self, AccountConfig};
use crate::error::{Error, Result};
use crate::output;
use crate::secrets::{self, SecretEntry};
use crate::{cli::LoginArgs, obj};

pub fn run(args: LoginArgs) -> Result<()> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(Error::invalid("Account name cannot be empty."));
    }

    let _lock = config::lock()?;
    let mut conf = config::load()?;

    if conf.find(&name).is_some() && !args.force {
        return Err(
            Error::invalid(format!("Account '{name}' already exists.")).fix(format!("spaceship login {name} --force"))
        );
    }

    let api_key = match (args.api_key, args.api_key_stdin) {
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

    let api_secret = match (args.api_secret, args.api_secret_stdin) {
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

    if !args.no_verify {
        let client = Client::new(&api_key, &api_secret, false);
        // Test credentials by requesting a 1-item domain list
        let _ = client.get("domains?take=1")?;
    }

    store.set(&secrets::account_key(&name), &SecretEntry { api_key: api_key.clone(), api_secret })?;

    let identity = if api_key.len() > 8 { format!("{}...", &api_key[..8]) } else { api_key };

    conf.accounts.insert(name.clone(), AccountConfig { identity: identity.clone(), added_at: config::now_utc() });
    config::save(&conf)?;

    output::write(&obj! {
        "status" => "logged_in",
        "name" => name,
        "identity" => identity,
        "secretStore" => store.name(),
    });

    Ok(())
}
