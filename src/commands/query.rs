use std::io::Read as _;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::cli::{Cli, OutputFormat, QueryArgs};
use crate::client::Client;
use crate::output;

pub fn run(client: &Client, cli: &Cli, args: &QueryArgs) -> Result<()> {
    let path = format!("/{}/_search", args.index);
    let value = if let Some(file) = &args.file {
        let text = if file == "-" {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .context("reading query body from stdin")?;
            text
        } else {
            std::fs::read_to_string(file).with_context(|| format!("reading {file}"))?
        };
        let mut body: Value = serde_json::from_str(&text).context("parsing query body as JSON")?;
        if let Some(size) = args.size {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("size".to_string(), Value::from(size));
            }
        }
        client.post_json(&path, &body)?
    } else {
        let mut params: Vec<(String, String)> = Vec::new();
        if let Some(query) = &args.query {
            params.push(("q".to_string(), query.clone()));
        }
        if let Some(size) = args.size {
            params.push(("size".to_string(), size.to_string()));
        }
        client.get_json(&path, &params)?
    };
    match cli.output {
        OutputFormat::Json => output::print_json(&value, cli.pretty),
        OutputFormat::Table => print!("{}", output::render_hits(&value)),
    }
    Ok(())
}
