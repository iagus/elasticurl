use anyhow::Result;

use crate::cli::{Cli, OutputFormat};
use crate::client::Client;
use crate::output;

pub fn run(client: &Client, cli: &Cli, index: &str) -> Result<()> {
    let value = client.get_json(&format!("/{index}/_mapping"), &[])?;
    match cli.output {
        OutputFormat::Json => output::print_json(&value, cli.pretty),
        OutputFormat::Table => {
            let rows = output::flatten_mappings(&value);
            if rows.is_empty() {
                println!("no mappings found");
            } else {
                print!("{}", output::render_table(&["field", "type"], &rows));
            }
        }
    }
    Ok(())
}
