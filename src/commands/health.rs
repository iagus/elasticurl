use anyhow::Result;

use crate::cli::{Cli, OutputFormat};
use crate::client::Client;
use crate::output;

pub fn run(client: &Client, cli: &Cli) -> Result<()> {
    let value = client.get_json("/_cluster/health", &[])?;
    match cli.output {
        OutputFormat::Json => output::print_json(&value, cli.pretty),
        OutputFormat::Table => print!("{}", output::render_kv(&value)),
    }
    Ok(())
}
