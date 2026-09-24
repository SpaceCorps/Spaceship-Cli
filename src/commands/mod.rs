//! Command execution and dispatch.

pub mod accounts;
pub mod contacts;
pub mod dns;
pub mod domains;
pub mod hyperlift;
pub mod login;
pub mod operations;
pub mod sellerhub;

use std::io::{IsTerminal, Read};

use serde_json::Value;

use crate::account;
use crate::cli::{AuthArgs, Command};
use crate::error::{Error, Result};
use crate::readme;

pub fn run(command: Command, auth: AuthArgs, verbose: bool) -> Result<()> {
    match command {
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
        Command::Login(args) => login::run(args),
        Command::Accounts(cmd) => accounts::run(cmd),
        Command::Domains(cmd) => {
            let client =
                account::resolve(auth.account.as_deref(), auth.api_key.as_deref(), auth.api_secret.as_deref())?
                    .client(verbose);
            domains::run(&client, cmd)
        }
        Command::Dns(cmd) => {
            let client =
                account::resolve(auth.account.as_deref(), auth.api_key.as_deref(), auth.api_secret.as_deref())?
                    .client(verbose);
            dns::run(&client, cmd)
        }
        Command::Contacts(cmd) => {
            let client =
                account::resolve(auth.account.as_deref(), auth.api_key.as_deref(), auth.api_secret.as_deref())?
                    .client(verbose);
            contacts::run(&client, cmd)
        }
        Command::Sellerhub(cmd) => {
            let client =
                account::resolve(auth.account.as_deref(), auth.api_key.as_deref(), auth.api_secret.as_deref())?
                    .client(verbose);
            sellerhub::run(&client, cmd)
        }
        Command::Hyperlift(cmd) => {
            let client =
                account::resolve(auth.account.as_deref(), auth.api_key.as_deref(), auth.api_secret.as_deref())?
                    .client(verbose);
            hyperlift::run(&client, cmd)
        }
        Command::Operations(cmd) => {
            let client =
                account::resolve(auth.account.as_deref(), auth.api_key.as_deref(), auth.api_secret.as_deref())?
                    .client(verbose);
            operations::run(&client, cmd)
        }
    }
}

pub fn read_json_input(file: Option<&str>, expected_hint: &str) -> Result<Value> {
    let content = if let Some(path) = file.filter(|p| !p.trim().is_empty()) {
        std::fs::read_to_string(path).map_err(|e| Error::invalid(format!("Failed to read file '{path}': {e}")))?
    } else if std::io::stdin().is_terminal() {
        return Err(Error::invalid(format!("Provide JSON via stdin or --file. Expected: {expected_hint}")));
    } else {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            return Err(Error::invalid(format!(
                "Stdin was empty. Provide JSON via stdin or --file. Expected: {expected_hint}"
            )));
        }
        buf
    };

    serde_json::from_str(&content).map_err(|e| Error::invalid(format!("Invalid JSON: {e}")))
}
