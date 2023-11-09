set dotenv-load
set positional-arguments

# List available recipes and their arguments
default: list

# List available recipes and their arguments
list:
    @just --list

# Install cargo watch if it's not already available
ensure-cargo-watch:
    @cargo install -q cargo-watch

# Install cargo watch if it's not already available
ensure-cargo-nextest:
    @cargo install -q cargo-nextest

# Use SQLa to generate bootstrap and code notebook SQL
sqla-sync:
    @./support/sql-aide/sqlactl.ts
    @rm -f ./bootstrap.sqlite.db
    @cat src/bootstrap.sql | sqlite3 ./bootstrap.sqlite.db && rm -f ./bootstrap.sqlite.db
    @# if there are any errors, ./bootstrap.sqlite.db should still be available

# Hot reload (with `cargo watch`) on all the source files in this project
dev: ensure-cargo-watch    
    @cargo watch -q -c -w src/ -x 'run -q'

# Run unit tests using cargo-nextest
test: ensure-cargo-nextest
    @cargo nextest run

# Lint all the code
lint:
    @cargo clippy

# Create the release build and prepare for tagging
release:
    @cargo build --release

# Generate CLI markdown help and save in CLI-help.md
help-markdown:
    @cargo run -- admin cli-help-md > CLI-help.md

# Pass all arguments to `cargo run --` to execute the CLI
run *args='--help':
    @cargo run -- $@
