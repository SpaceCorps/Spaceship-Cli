//! Account credential resolution and client creation.
//!
//! Credentials can come from:
//! 1. A named account in the OS keystore via `--account <name>` (or `-a <name>`).
//! 2. Direct CLI flags `--api-key` and `--api-secret`.
//! 3. Environment variables `SPACESHIP_API_KEY` and `SPACESHIP_API_SECRET`.

use crate::client::Client;
use crate::config::{self, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct Resolved {
    #[allow(dead_code)]
    pub name: String,
    pub api_key: String,
    pub api_secret: String,
}

impl Resolved {
    pub fn client(&self, verbose: bool) -> Client {
        Client::new(&self.api_key, &self.api_secret, verbose)
    }
}

pub fn resolve(requested: Option<&str>, flag_key: Option<&str>, flag_secret: Option<&str>) -> Result<Resolved> {
    let config = config::load()?;

    // 1. If an explicit account was requested, resolve strictly from the keystore
    if let Some(requested) = requested.map(str::trim).filter(|s| !s.is_empty()) {
        let Some((name, _account)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'."))
                .detail(describe(&config))
                .fix("spaceship accounts list"));
        };

        let creds = secrets::store()?.get(&secrets::account_key(name))?;
        let Some(creds) = creds.filter(|c| !c.api_key.trim().is_empty() && !c.api_secret.trim().is_empty()) else {
            return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{name}' has no stored credentials."))
                .detail("The config entry exists but the keystore has nothing under it.")
                .fix(format!("spaceship accounts add {name} --api-key <key> --api-secret <secret>")));
        };

        return Ok(Resolved { name: name.clone(), api_key: creds.api_key, api_secret: creds.api_secret });
    }

    // 2. Check explicit flags
    let flag_k = flag_key.map(str::trim).filter(|s| !s.is_empty());
    let flag_s = flag_secret.map(str::trim).filter(|s| !s.is_empty());
    if let (Some(k), Some(s)) = (flag_k, flag_s) {
        return Ok(Resolved { name: "(flags)".into(), api_key: k.to_string(), api_secret: s.to_string() });
    }

    // 3. Check environment variables
    let env_k = std::env::var("SPACESHIP_API_KEY").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let env_s = std::env::var("SPACESHIP_API_SECRET").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    if let (Some(k), Some(s)) = (env_k, env_s) {
        return Ok(Resolved { name: "(environment)".into(), api_key: k, api_secret: s });
    }

    if flag_k.is_some() || flag_s.is_some() {
        return Err(Error::invalid("Both --api-key and --api-secret are required when passing credentials via flags."));
    }

    Err(Error::new(
        ErrorCode::NoAccount,
        "No account specified. Pass --account <name> or set SPACESHIP_API_KEY and SPACESHIP_API_SECRET.",
    )
    .detail(describe(&config))
    .fix("spaceship accounts list"))
}

fn describe(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No accounts are configured yet. Run 'spaceship accounts add <name> --api-key <key> --api-secret <secret>'.".into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| if v.identity.trim().is_empty() { k.clone() } else { format!("{k} ({})", v.identity) })
        .collect();
    format!("Configured accounts: {}", listed.join(", "))
}
