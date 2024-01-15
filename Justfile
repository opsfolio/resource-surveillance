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

# Hot reload (with `cargo watch`) on all the source files in this project
dev: ensure-cargo-watch    
    @cargo watch -q -c -w src/ -x 'run -q'

# Run unit tests using cargo-nextest
test: 
    @cargo test -- --test-threads=1

# Run end-to-end tests
test-e2e: 
    rm -f ./e2e-test-state.sqlite.db && just sqla-sync && just run --debug ingest files -d ./e2e-test-state.sqlite.db --stats --save-behavior behavior1

# Run end-to-end tests only in support/test-fixtures
test-e2e-fixtures: 
    rm -f ./e2e-test-state.sqlite.db && just sqla-sync && just run --debug ingest files -d ./e2e-test-state.sqlite.db -r ./support/test-fixtures --stats

# Lint all the code
lint:
    @cargo clippy -- -D warnings

# Format the code:
fmt:
    @cargo fmt

# Create the release build and prepare for tagging
release:
    @cargo build --release

# Generate CLI markdown help and save in CLI-help.md
help-markdown:
    @cargo run -- admin cli-help-md > support/docs/CLI-help.md

# Generate `tbls` database schema documents
tbls: sqla-sync
    @just run admin init -d tbls-temp.sqlite.db
    @tbls -c ./support/docs/surveilr-code-notebooks.tbls.auto.yml doc sqlite://./tbls-temp.sqlite.db ./support/docs/surveilr-code-notebooks-schema --rm-dist
    @tbls -c ./support/docs/surveilr-state.tbls.auto.yml doc sqlite://./tbls-temp.sqlite.db ./support/docs/surveilr-state-schema --rm-dist
    @rm -f ./tbls-temp.sqlite.db

# Pass all arguments to `cargo run --` to execute the CLI
run *args='--help':
    @cargo run -- $@

# Perform all the functions to prepare for a release
prepare-release: lint help-markdown tbls test test-e2e