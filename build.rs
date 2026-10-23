// Generates one man page per (sub)command into `man/` using clap_mangen, so
// the man pages always match `--help`. Note: this writes into the source
// directory on purpose (they are .gitignored and meant for `make`-less
// installation via `man -l` or copying into a manpath).

use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

// Reuse the CLI definition as the single source for help docs.
include!("src/cli.rs");

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=src/cli.rs");
    println!("cargo::rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("man");
    fs::create_dir_all(&out_dir)?;

    let mut cmd = <Cli as clap::CommandFactory>::command();
    cmd.build();
    clap_mangen::generate_to(cmd, &out_dir)?;

    // Drop man pages for clap's auto-generated `help` subcommand.
    for entry in fs::read_dir(&out_dir)? {
        let path = entry?.path();
        let is_help_page = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains("-help"));
        if is_help_page {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}
