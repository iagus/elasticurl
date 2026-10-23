use clap::Parser;
use elasticurl::cli::Cli;
use elasticurl::client::Client;
use elasticurl::commands;

fn main() {
    let cli = Cli::parse();
    let client = Client::new(&cli.host, cli.user.clone(), cli.verbose);
    if let Err(err) = commands::run(&cli, &client) {
        eprintln!("elasticurl: error: {err:#}");
        std::process::exit(1);
    }
}
