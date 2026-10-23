// CLI argument definitions — the single source of truth for `--help` and the
// generated man pages (build.rs includes this file).

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Output format for command results.
#[derive(Copy, Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table (Elasticsearch `_cat` output where possible)
    #[default]
    Table,
    /// Machine-readable JSON (combine with --pretty)
    Json,
}

/// Simplify common requests to a (local) Elasticsearch API.
///
/// elasticurl wraps the most common Elasticsearch endpoints so you don't have
/// to memorize their paths: list indices and aliases, inspect mappings, check
/// cluster health and index sizes, or query an index.
///
/// All commands print a table by default; use `-o json` for machine-readable
/// output and `--pretty` to format it.
#[derive(Parser, Debug)]
#[command(name = "elasticurl", version)]
pub struct Cli {
    /// Elasticsearch base URL
    #[arg(
        short = 'H',
        long,
        env = "ELASTIC_URL",
        default_value = "http://localhost:9200",
        global = true,
        value_name = "URL"
    )]
    pub host: String,

    /// Basic auth credentials
    #[arg(
        short,
        long,
        env = "ELASTIC_AUTH",
        global = true,
        value_name = "USER:PASS"
    )]
    pub user: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    pub output: OutputFormat,

    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pub pretty: bool,

    /// Print request/response details to stderr
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Show cluster health: status, node count and shard counts
    Health,

    /// List all indices
    Indices,

    /// Show index sizes (docs, store size), biggest first
    Size {
        /// Restrict to a single index
        index: Option<String>,
    },

    /// Show the field mappings of an index
    Mappings {
        /// Index name
        index: String,
    },

    /// List all aliases and the indices they point to
    Aliases,

    /// Search an index with a Lucene query (-q) or a JSON DSL body (-f)
    Query(QueryArgs),

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Index to search
    pub index: String,

    /// Lucene query string, e.g. 'status:404'. Defaults to match_all
    #[arg(short, long, value_name = "QUERY", conflicts_with = "file")]
    pub query: Option<String>,

    /// File with a JSON query DSL body ('-' reads from stdin)
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<String>,

    /// Maximum number of hits to return
    #[arg(short = 'n', long, value_name = "N")]
    pub size: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults() {
        let cli = Cli::try_parse_from(["elasticurl", "health"]).unwrap();
        assert_eq!(cli.host, "http://localhost:9200");
        assert!(matches!(cli.output, OutputFormat::Table));
        assert!(matches!(cli.command, Command::Health));
    }

    #[test]
    fn parses_global_flags_after_subcommand() {
        let cli =
            Cli::try_parse_from(["elasticurl", "indices", "-o", "json", "-H", "http://es:9200"])
                .unwrap();
        assert_eq!(cli.host, "http://es:9200");
        assert!(matches!(cli.output, OutputFormat::Json));
    }

    #[test]
    fn query_conflict_between_q_and_file() {
        assert!(Cli::try_parse_from(["elasticurl", "query", "logs", "-q", "a:b", "-f", "b.json"])
            .is_err());
    }

    #[test]
    fn query_accepts_lucene() {
        let cli =
            Cli::try_parse_from(["elasticurl", "query", "logs", "-q", "status:404", "-n", "5"])
                .unwrap();
        let Command::Query(args) = cli.command else {
            panic!("expected query command")
        };
        assert_eq!(args.index, "logs");
        assert_eq!(args.query.as_deref(), Some("status:404"));
        assert_eq!(args.size, Some(5));
    }

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
