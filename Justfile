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

# Install tarpaulin for code coverage
ensure-cargo-tarpaulin:
    @cargo install -q cargo-tarpaulin

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
    @cargo test --lib --bins -- --test-threads=1

# Run end-to-end tests
test-e2e: 
    rm -f ./e2e-test-state.sqlite.db && just sqla-sync && just run --debug ingest files -d ./e2e-test-state.sqlite.db --stats --save-behavior behavior1

# Run end-to-end tests only in support/test-fixtures
test-e2e-fixtures: 
    rm -f ./e2e-test-state.sqlite.db && just sqla-sync && just run --debug ingest files -d ./e2e-test-state.sqlite.db -r ./support/test-fixtures --stats

# Run the whole regression test suite.
test-regression:
    just test-regression-ingest && just test-regression-imap

# Run the regression tests for files ingestion
test-regression-ingest:
    rm -f ./regression-ingest-files.sqlite.db && just run --debug ingest files -d ./regression-ingest-files.sqlite.db -r ./support/test-fixtures --stats
    cat ./support/regression-tests/ingest-files.sql | sqlite3 ./regression-ingest-files.sqlite.db

# Run the regression tests for ingesting from an email using IMAP
test-regression-imap:
    rm -f ./regression-ingest-imap.sqlite.db
    cargo run -- ingest imap -d ./regression-ingest-imap.sqlite.db -u surveilrregression@gmail.com --password 'ingq hidi atao zrka' -a "imap.gmail.com" -b 10 --css-select "all-png-images:img[src$='.png']" --css-select "mailchimp-urls:a[href*='mailchimp.com']"
    cat ./support/regression-tests/ingest-imap.sql | sqlite3 ./regression-ingest-imap.sqlite.db

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

# Generates local code coverage data
code-coverage: ensure-cargo-tarpaulin
    @cargo +nightly tarpaulin --verbose --all-features --workspace --timeout 120 --out xml -- --test-threads=1

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