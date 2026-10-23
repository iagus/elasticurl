# elasticurl

A small CLI that simplifies common requests to a local (or remote)
Elasticsearch API — no need to memorize endpoint paths.

Written in Rust: a single ~2 MB static binary with instant startup, and man
pages plus shell completions generated from the same source as `--help`.

## Commands

| Command | Elasticsearch request |
|---|---|
| `elasticurl health` | `GET /_cluster/health` |
| `elasticurl indices` | `GET /_cat/indices?v` |
| `elasticurl size [index]` | `GET /_cat/indices[/<index>]`, biggest first |
| `elasticurl mappings <index>` | `GET /<index>/_mapping` |
| `elasticurl aliases` | `GET /_cat/aliases?v` |
| `elasticurl query <index> [-q \| -f]` | `GET`/`POST /<index>/_search` |

## Global flags

| Flag | Env var | Default | Description |
|---|---|---|---|
| `-H, --host <URL>` | `ELASTIC_URL` | `http://localhost:9200` | Elasticsearch base URL |
| `-u, --user <USER:PASS>` | `ELASTIC_AUTH` | — | Basic auth credentials |
| `-o, --output <FMT>` | | `table` | `table` or `json` |
| `--pretty` | | off | Pretty-print JSON output |
| `-v, --verbose` | | off | Print request/response lines to stderr |

## Examples

```sh
elasticurl health
elasticurl indices
elasticurl indices -o json | jq '.[].index'
elasticurl size                    # all indices, biggest first
elasticurl size logs-2026.10
elasticurl mappings logs           # flattened field/type table
elasticurl aliases
elasticurl query logs                          # match_all, default size
elasticurl query logs -q 'status:404' -n 5
elasticurl query logs -f query.json
cat query.json | elasticurl query logs -f -    # body from stdin
elasticurl query logs -q 'msg:error' -o json --pretty
```

## Install

```sh
cargo install --path .
# or:
cargo build --release   # binary: target/release/elasticurl
```

## Man pages

`build.rs` generates a man page for `elasticurl` and every subcommand into
`man/` at build time (they always match `--help`):

```sh
man -l man/elasticurl.1
man -l man/elasticurl-query.1
# or install system-wide:
sudo install -m 644 man/*.1 /usr/local/share/man/man1/
```

## Shell completions

```sh
elasticurl completions bash > /etc/bash_completion.d/elasticurl
elasticurl completions zsh  > "${fpath[1]}/_elasticurl"
elasticurl completions fish > ~/.config/fish/completions/elasticurl.fish
```

## Development

```sh
cargo build
cargo test
```

The release profile is tuned for binary size (`opt-level = "z"`, LTO,
`codegen-units = 1`, `panic = "abort"`, `strip`).
