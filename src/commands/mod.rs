mod aliases;
mod completions;
mod health;
mod indices;
mod mappings;
mod query;
mod size;

use anyhow::Result;

use crate::cli::{Cli, Command};
use crate::client::Client;

pub fn run(cli: &Cli, client: &Client) -> Result<()> {
    match &cli.command {
        Command::Health => health::run(client, cli),
        Command::Indices => indices::run(client, cli),
        Command::Size { index } => size::run(client, cli, index.as_deref()),
        Command::Mappings { index } => mappings::run(client, cli, index),
        Command::Aliases => aliases::run(client, cli),
        Command::Query(args) => query::run(client, cli, args),
        Command::Completions { shell } => completions::run(*shell),
    }
}
