//! Hyperlift command implementations.

use serde_json::{Value, json};

use crate::cli::{HyperliftCommand, HyperliftEnvCommand};
use crate::client::{Client, query, seg};
use crate::commands::read_json_input;
use crate::error::{Error, Result};
use crate::output;

pub fn run(client: &Client, cmd: HyperliftCommand) -> Result<()> {
    match cmd {
        HyperliftCommand::List { take, skip } => list(client, take, skip),
        HyperliftCommand::Get { id } => get(client, &id),
        HyperliftCommand::Build { id } => build(client, &id),
        HyperliftCommand::BuildLogs { id, take, cursor } => build_logs(client, &id, take, cursor),
        HyperliftCommand::Logs { id, take, cursor } => logs(client, &id, take, cursor),
        HyperliftCommand::Metrics { id, start, end, interval, metrics } => {
            metrics_cmd(client, &id, &start, &end, &interval, &metrics)
        }
        HyperliftCommand::Restart { id } => restart(client, &id),
        HyperliftCommand::Scale { id, scale } => scale_cmd(client, &id, scale),
        HyperliftCommand::Env(sub) => env_cmd(client, sub),
    }
}

fn list(client: &Client, take: u32, skip: u32) -> Result<()> {
    let q = query(&[("take", Some(take.to_string())), ("skip", Some(skip.to_string()))]);
    let path = format!("hyperlift/applications{q}");
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn get(client: &Client, id: &str) -> Result<()> {
    let path = format!("hyperlift/applications/{}", seg(id));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn build(client: &Client, id: &str) -> Result<()> {
    let path = format!("hyperlift/applications/{}/build", seg(id));
    let res = client.post(&path, None)?;
    output::write(&res);
    Ok(())
}

fn build_logs(client: &Client, id: &str, take: u32, cursor: Option<String>) -> Result<()> {
    if !(1..=100).contains(&take) {
        return Err(Error::invalid("--take must be between 1 and 100."));
    }
    let q = query(&[("take", Some(take.to_string())), ("cursor", cursor)]);
    let path = format!("hyperlift/applications/{}/build-logs{q}", seg(id));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn logs(client: &Client, id: &str, take: u32, cursor: Option<String>) -> Result<()> {
    if !(1..=100).contains(&take) {
        return Err(Error::invalid("--take must be between 1 and 100."));
    }
    let q = query(&[("take", Some(take.to_string())), ("cursor", cursor)]);
    let path = format!("hyperlift/applications/{}/logs{q}", seg(id));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn metrics_cmd(client: &Client, id: &str, start: &str, end: &str, interval: &str, metrics: &str) -> Result<()> {
    let q = query(&[
        ("startDate", Some(start.to_string())),
        ("endDate", Some(end.to_string())),
        ("interval", Some(interval.to_string())),
        ("metrics", Some(metrics.to_string())),
    ]);
    let path = format!("hyperlift/applications/{}/metrics{q}", seg(id));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn restart(client: &Client, id: &str) -> Result<()> {
    let path = format!("hyperlift/applications/{}/restart", seg(id));
    let res = client.post(&path, None)?;
    output::write(&res);
    Ok(())
}

fn scale_cmd(client: &Client, id: &str, scale: u32) -> Result<()> {
    if scale > 1 {
        return Err(Error::invalid("--scale must be 0 (stop) or 1 (start)."));
    }
    let body = json!({ "scale": scale });
    let path = format!("hyperlift/applications/{}/scale", seg(id));
    let res = client.put(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

fn env_cmd(client: &Client, cmd: HyperliftEnvCommand) -> Result<()> {
    match cmd {
        HyperliftEnvCommand::Get { id } => {
            let path = format!("hyperlift/applications/{}/environment", seg(&id));
            let res = client.get(&path)?;
            output::write(&res);
            Ok(())
        }
        HyperliftEnvCommand::Set { id, file } => {
            let body = read_json_input(file.as_deref(), "{\"NODE_ENV\": \"production\"} or {\"items\": {...}}")?;
            if !body.is_object() {
                return Err(Error::invalid("Expected a JSON object mapping variable names to values."));
            }

            let payload = match &body {
                Value::Object(map) if map.contains_key("items") && map.get("items").unwrap().is_object() => body,
                _ => json!({ "items": body }),
            };

            let path = format!("hyperlift/applications/{}/environment", seg(&id));
            let res = client.put(&path, Some(&payload))?;
            output::write(&res);
            Ok(())
        }
    }
}
