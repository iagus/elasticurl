use anyhow::Result;

use crate::cli::{Cli, OutputFormat};
use crate::client::Client;
use crate::output;

pub fn run(client: &Client, cli: &Cli, index: Option<&str>) -> Result<()> {
    let path = match index {
        Some(index) => format!("/_cat/indices/{index}"),
        None => "/_cat/indices".to_string(),
    };
    let mut params = vec![
        ("v".to_string(), "true".to_string()),
        (
            "h".to_string(),
            "health,status,index,docs.count,store.size,pri.store.size".to_string(),
        ),
        ("s".to_string(), "store.size:desc".to_string()),
    ];
    if matches!(cli.output, OutputFormat::Json) {
        params.push(("format".to_string(), "json".to_string()));
        let value = client.get_json(&path, &params)?;
        output::print_json(&value, cli.pretty);
    } else {
        let body = client.get_text(&path, &params)?;
        output::print_text(&body);
    }
    Ok(())
}
