set dotenv-load
set positional-arguments

default: list

# List available recipes and their arguments
list:
    @just --list

# Install sea-orm-cli if it's not already available
ensure-sea-orm-cli:
    @cargo install -q sea-orm-cli

# Install cargo watch if it's not already available
ensure-cargo-watch:
    @cargo install -q cargo-watch

# Generate entity files of database `device-surveillance.sqlite.db` to `src/entities.auto`
orm-sync: ensure-sea-orm-cli
    @sea-orm-cli generate entity -u "sqlite://./device-surveillance.sqlite.db" -o "src/entities.auto"

# Hot reload (with `cargo watch`) on all the source files in this project
dev: ensure-cargo-watch    
    @cargo watch -q -c -w src/ -x 'run -q'

# Generate CLI markdown help and save in CLI-help.md
help-markdown:
    @cargo run -- --help-markdown > CLI-help.md

# Pass all arguments to `cargo run --` to execute the CLI
run *args='--help':
    @cargo run -- $@
