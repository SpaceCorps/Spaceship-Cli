//! Async operations command implementations.

use crate::cli::OperationsCommand;
use crate::client::{Client, seg};
use crate::error::Result;
use crate::output;

pub fn run(client: &Client, cmd: OperationsCommand) -> Result<()> {
    match cmd {
        OperationsCommand::Get { id } => {
            let path = format!("async-operations/{}", seg(&id));
            let res = client.get(&path)?;
            output::write(&res);
            Ok(())
        }
    }
}
