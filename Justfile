set dotenv-load
set positional-arguments

# List available recipes and their arguments
default: list

# List available recipes and their arguments
list:
    @just --list

# Install cargo watch if it's not already available
ensure-cargo-nextest:
    @cargo install -q cargo-nextest

# Install cargo watch if it's not already available
ensure-cargo-sbom:
    @cargo install -q cargo-watch

# Install cargo SBOM if it's not already available
ensure-cargo-watch:
    @cargo install -q cargo-sbom

# Generate surveilr binary SBOM in SPDX format
sbom: ensure-cargo-sbom
    @cargo-sbom > support/quality-system/surveilr-sbom.spdx.json

# Use SQLa to generate bootstrap and code notebook SQL
sqla-sync:
    @./support/sql-aide/sqlactl.ts
    @rm -f ./sqla-sync-state.sqlite.db
    @cat src/bootstrap.sql | sqlite3 ./sqla-sync-state.sqlite.db && rm -f ./sqla-sync-state.sqlite.db
    @# if there are any errors, ./sqla-sync-state.sqlite.db should still be available

# Hot reload (with `cargo watch`) on all the source files in this project
dev: ensure-cargo-watch    
    @cargo watch -q -c -w src/ -x 'run -q'

# Run unit tests using cargo-nextest
test: ensure-cargo-nextest
    @cargo nextest run

# Run end-to-end tests
test-e2e: 
    rm -f ./e2e-test-state.sqlite.db && just sqla-sync && just run --debug fs-walk -d ./e2e-test-state.sqlite.db --stats

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
