# Development and Contributors Guide

**IMPORTANT**: Use SQLa to generate all SQL so it's portable but use Rusqlite to
make working with SQLite more ergonomic. Remember to only use libraries to help
improve developer productivity, always assume SQLite database will be used
across polyglot programming environments so SQL code should be transparent and
portable.

Development prerequisites:

- Install Rust toolchain (1.73 or above, best to use `rustup`, `asdf` or `rtx`
  for multiple simultaneous versions)
- `cargo install just` so we can use `Justfile` for task management

Regular use:

```bash
$ just --completions fish | source            # setup completions to reduce typing

$ just test                                   # run unit tests with cargo nextest

$ just run                                    # get CLI help
$ cargo run -- --help                         # get CLI help, same as above

$ just run admin cli-help-md                  # get CLI in Markdown and update this README.md manually
$ cargo run -- --help-markdown > CLI-help.md  # get CLI in Markdown, same as above

$ just run fs-walk --help                     # get CLI help for fs-walk subcommand
$ just run --debug fs-walk                    # walk the current working directory (CWD) with debug messages
$ just run fs-walk -r /other -r /other2       # walk some other director(ies)
$ just run fs-walk -i .git/                   # walk CWD, ignore .git/ paths
$ just run fs-walk -i .git/ -i target/        # walk CWD, ignore .git/ and target/ paths

$ just sqla-sync                              # generate SQLa bootstrap and other SQL
                                             
$ just dev                                    # turn on auto-compile, auto-run during development
                                              # using cargo-watch command
```
