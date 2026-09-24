//! Contacts command implementations.

use serde_json::Value;

use crate::cli::{ContactAttributesCommand, ContactsCommand};
use crate::client::{Client, seg};
use crate::commands::read_json_input;
use crate::error::Result;
use crate::output;

pub fn run(client: &Client, cmd: ContactsCommand) -> Result<()> {
    match cmd {
        ContactsCommand::Save {
            first_name,
            last_name,
            email,
            address,
            city,
            country,
            phone,
            phone_ext,
            fax,
            fax_ext,
            tax_number,
            organization,
            address2,
            state,
            postal_code,
        } => save(
            client,
            &first_name,
            &last_name,
            &email,
            &address,
            &city,
            &country,
            &phone,
            phone_ext.as_deref(),
            fax.as_deref(),
            fax_ext.as_deref(),
            tax_number.as_deref(),
            organization.as_deref(),
            address2.as_deref(),
            state.as_deref(),
            postal_code.as_deref(),
        ),
        ContactsCommand::Get { id } => get(client, &id),
        ContactsCommand::Attributes(sub) => attributes(client, sub),
    }
}

#[allow(clippy::too_many_arguments)]
fn save(
    client: &Client,
    first_name: &str,
    last_name: &str,
    email: &str,
    address: &str,
    city: &str,
    country: &str,
    phone: &str,
    phone_ext: Option<&str>,
    fax: Option<&str>,
    fax_ext: Option<&str>,
    tax_number: Option<&str>,
    organization: Option<&str>,
    address2: Option<&str>,
    state: Option<&str>,
    postal_code: Option<&str>,
) -> Result<()> {
    let mut map = serde_json::Map::new();
    map.insert("firstName".into(), Value::String(first_name.into()));
    map.insert("lastName".into(), Value::String(last_name.into()));
    map.insert("email".into(), Value::String(email.into()));
    map.insert("address1".into(), Value::String(address.into()));
    map.insert("city".into(), Value::String(city.into()));
    map.insert("country".into(), Value::String(country.into()));
    map.insert("phone".into(), Value::String(phone.into()));

    if let Some(v) = phone_ext {
        map.insert("phoneExt".into(), Value::String(v.into()));
    }
    if let Some(v) = fax {
        map.insert("fax".into(), Value::String(v.into()));
    }
    if let Some(v) = fax_ext {
        map.insert("faxExt".into(), Value::String(v.into()));
    }
    if let Some(v) = tax_number {
        map.insert("taxNumber".into(), Value::String(v.into()));
    }
    if let Some(v) = organization {
        map.insert("organization".into(), Value::String(v.into()));
    }
    if let Some(v) = address2 {
        map.insert("address2".into(), Value::String(v.into()));
    }
    if let Some(v) = state {
        map.insert("stateProvince".into(), Value::String(v.into()));
    }
    if let Some(v) = postal_code {
        map.insert("postalCode".into(), Value::String(v.into()));
    }

    let res = client.put("contacts", Some(&Value::Object(map)))?;
    output::write(&res);
    Ok(())
}

fn get(client: &Client, id: &str) -> Result<()> {
    let path = format!("contacts/{}", seg(id));
    let res = client.get(&path)?;
    output::write(&res);
    Ok(())
}

fn attributes(client: &Client, cmd: ContactAttributesCommand) -> Result<()> {
    match cmd {
        ContactAttributesCommand::Save { file } => {
            let body = read_json_input(file.as_deref(), "{\"type\": \"...\", ...}")?;
            let res = client.put("contacts/attributes", Some(&body))?;
            output::write(&res);
            Ok(())
        }
        ContactAttributesCommand::Get { id } => {
            let path = format!("contacts/attributes/{}", seg(&id));
            let res = client.get(&path)?;
            output::write(&res);
            Ok(())
        }
    }
}
