//! DNS records command implementations.

use serde_json::Value;

use crate::client::{Client, query, seg};
use crate::commands::read_json_input;
use crate::error::{Error, Result};
use crate::output;
use crate::{cli::DnsCommand, obj};

pub fn run(client: &Client, cmd: DnsCommand) -> Result<()> {
    match cmd {
        DnsCommand::List { domain, take, skip, all, order_by, record_type, name } => {
            list(client, &domain, take, skip, all, order_by, record_type, name)
        }
        DnsCommand::Save { domain, file, force } => save(client, &domain, file.as_deref(), force),
        DnsCommand::Delete { domain, file } => delete(client, &domain, file.as_deref()),
    }
}

#[allow(clippy::too_many_arguments)]
fn list(
    client: &Client,
    domain: &str,
    take: u32,
    skip: u32,
    all: bool,
    order_by: Option<String>,
    record_type: Option<String>,
    name: Option<String>,
) -> Result<()> {
    if !(1..=500).contains(&take) {
        return Err(Error::invalid("--take must be between 1 and 500."));
    }

    let filtering = record_type.is_some() || name.is_some();
    let base_path = format!("dns/records/{}", seg(domain));

    if !all && !filtering {
        let q = query(&[("take", Some(take.to_string())), ("skip", Some(skip.to_string())), ("orderBy", order_by)]);
        let res = client.get(&format!("{base_path}{q}"))?;
        output::write(&res);
        return Ok(());
    }

    // Page through the zone and filter locally
    const PAGE_SIZE: u32 = 500;
    let mut current_skip = skip;
    let mut all_records: Vec<Value> = Vec::new();

    loop {
        let q = query(&[
            ("take", Some(PAGE_SIZE.to_string())),
            ("skip", Some(current_skip.to_string())),
            ("orderBy", order_by.clone()),
        ]);
        let page = client.get(&format!("{base_path}{q}"))?;

        let items = page.get("items").and_then(Value::as_array).cloned().unwrap_or_default();
        let batch_count = items.len() as u32;
        all_records.extend(items);
        current_skip += batch_count;

        let total = page.get("total").and_then(Value::as_u64).map(|n| n as u32).unwrap_or(current_skip);

        if batch_count == 0 || current_skip >= total {
            break;
        }
    }

    let matches: Vec<Value> = all_records
        .into_iter()
        .filter(|r| matches_property(r, "type", record_type.as_deref()) && matches_property(r, "name", name.as_deref()))
        .collect();

    output::write(&obj! {
        "items" => Value::Array(matches.clone()),
        "total" => matches.len(),
    });

    Ok(())
}

fn matches_property(record: &Value, prop: &str, expected: Option<&str>) -> bool {
    let Some(exp) = expected.map(str::trim).filter(|s| !s.is_empty()) else {
        return true;
    };
    record.get(prop).and_then(Value::as_str).map(|v| v.eq_ignore_ascii_case(exp)).unwrap_or(false)
}

fn save(client: &Client, domain: &str, file: Option<&str>, force: bool) -> Result<()> {
    let body = read_json_input(file, "[ ... ] or {\"items\": [...]}")?;

    let mut payload = match body {
        Value::Array(arr) => {
            let mut map = serde_json::Map::new();
            map.insert("items".into(), Value::Array(arr));
            map
        }
        Value::Object(map) if map.contains_key("items") => map,
        _ => return Err(Error::invalid("Expected a JSON array of records, or {\"items\": [...]}.")),
    };

    if force {
        payload.insert("force".into(), Value::Bool(true));
    }

    let path = format!("dns/records/{}", seg(domain));
    let res = client.put(&path, Some(&Value::Object(payload)))?;
    output::write(&res);
    Ok(())
}

fn delete(client: &Client, domain: &str, file: Option<&str>) -> Result<()> {
    let body = read_json_input(file, "[ ... ] or {\"items\": [...]}")?;

    let payload = match body {
        Value::Array(arr) => Value::Array(arr),
        Value::Object(map) => match map.get("items") {
            Some(Value::Array(arr)) => Value::Array(arr.clone()),
            _ => return Err(Error::invalid("Expected a JSON array of records, or {\"items\": [...]}.")),
        },
        _ => return Err(Error::invalid("Expected a JSON array of records, or {\"items\": [...]}.")),
    };

    let path = format!("dns/records/{}", seg(domain));
    let res = client.delete(&path, Some(&payload))?;
    output::write(&res);
    Ok(())
}
