use anyhow::Result;

use crate::cli::{Cli, OutputFormat};
use crate::client::Client;
use crate::output;

pub fn run(client: &Client, cli: &Cli) -> Result<()> {
    let mut params = vec![("v".to_string(), "true".to_string())];
    if matches!(cli.output, OutputFormat::Json) {
        params.push(("format".to_string(), "json".to_string()));
        let value = client.get_json("/_cat/indices", &params)?;
        output::print_json(&value, cli.pretty);
    } else {
        let body = client.get_text("/_cat/indices", &params)?;
        output::print_text(&body);
    }
    Ok(())
}
