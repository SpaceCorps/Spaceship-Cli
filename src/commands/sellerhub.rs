//! SellerHub command implementations.

use crate::cli::{SafepayCommand, SellerhubCommand};
use crate::client::{Client, query, seg};
use crate::commands::read_json_input;
use crate::error::{Error, Result};
use crate::output;

pub fn run(client: &Client, cmd: SellerhubCommand) -> Result<()> {
    match cmd {
        SellerhubCommand::List { take, skip } => list(client, take, skip),
        SellerhubCommand::Get { domain } => get(client, &domain),
        SellerhubCommand::Create { file } => create(client, file.as_deref()),
        SellerhubCommand::Update { domain, file } => update(client, &domain, file.as_deref()),
        SellerhubCommand::Delete { domain } => delete(client, &domain),
        SellerhubCommand::Checkout { file } => checkout(client, file.as_deref()),
        SellerhubCommand::Verification => verification(client),
        SellerhubCommand::Sold { take, cursor, from, to } => sold(client, take, cursor, from, to),
        SellerhubCommand::Safepay(sub) => safepay(client, sub),
    }
}

fn list(client: &Client, take: u32, skip: u32) -> Result<()> {
    let q = query(&[("take", Some(take.to_string())), ("skip", Some(skip.to_string()))]);
    let path = format!("sellerhub/domains{q}");
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn get(client: &Client, domain: &str) -> Result<()> {
    let path = format!("sellerhub/domains/{}", seg(domain));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn create(client: &Client, file: Option<&str>) -> Result<()> {
    let body = read_json_input(file, "{\"domainName\": \"...\", ...}")?;
    let res = client.post("sellerhub/domains", Some(&body))?;
    output::write(&res);
    Ok(())
}

fn update(client: &Client, domain: &str, file: Option<&str>) -> Result<()> {
    let body = read_json_input(file, "{\"displayName\": \"...\", ...}")?;
    let path = format!("sellerhub/domains/{}", seg(domain));
    let res = client.patch(&path, Some(&body))?;
    output::write(&res);
    Ok(())
}

fn delete(client: &Client, domain: &str) -> Result<()> {
    let path = format!("sellerhub/domains/{}", seg(domain));
    let res = client.delete(&path, None)?;
    output::write(&res);
    Ok(())
}

fn checkout(client: &Client, file: Option<&str>) -> Result<()> {
    let body = read_json_input(file, "{\"type\": \"buyNow\", \"domainName\": \"...\", ...}")?;
    let res = client.post("sellerhub/checkout-links", Some(&body))?;
    output::write(&res);
    Ok(())
}

fn verification(client: &Client) -> Result<()> {
    let res = client.get("sellerhub/verification-records")?;
    output::write(&res);
    Ok(())
}

fn sold(client: &Client, take: u32, cursor: Option<String>, from: Option<String>, to: Option<String>) -> Result<()> {
    if !(1..=100).contains(&take) {
        return Err(Error::invalid("--take must be between 1 and 100."));
    }

    let q = query(&[
        ("take", Some(take.to_string())),
        ("cursor", cursor),
        ("saleDateTimeFrom", from),
        ("saleDateTimeTo", to),
    ]);
    let path = format!("sellerhub/domains/reports/sold{q}");
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn safepay(client: &Client, cmd: SafepayCommand) -> Result<()> {
    match cmd {
        SafepayCommand::List { take, skip } => {
            let q = query(&[("take", Some(take.to_string())), ("skip", Some(skip.to_string()))]);
            let path = format!("sellerhub/safepay-transactions{q}");
            let res = client.get(&path)?;
            output::write(&res);
            Ok(())
        }
        SafepayCommand::Get { transaction_id } => {
            let path = format!("sellerhub/safepay-transactions/{}", seg(&transaction_id));
            let res = client.get(&path)?;
            output::write(&res);
            Ok(())
        }
        SafepayCommand::Create { file } => {
            let body = read_json_input(
                file.as_deref(),
                "{\"domainName\": \"...\", \"initiatedBy\": \"seller\", \"basePrice\": {...}, \"type\": \"buyNow\", \"feePercentageShare\": {...}}",
            )?;
            let res = client.post("sellerhub/safepay-transactions", Some(&body))?;
            output::write(&res);
            Ok(())
        }
    }
}
