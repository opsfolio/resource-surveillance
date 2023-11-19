<p align="center">
  <img src="support/surveilr-logo-with-text-264x66px.png" width="264" height="66"/>
</p>

`surveilr` is an extendable file system inspector for performing surveillance of
machine resources. It is used to walk resources like file systems and generate
an SQLite database which can then be consumed by any computing environment that
supports SQLite.

The SQLite file which `surveilr` prepares is called the _Resource Surveillance
State Database_ (`RSSD`) and once it's been generated it's no longer tied to
`surveilr` and can be used by any other tool, service, application, or ETL'd
into a data warehouse.

![Architecture](support/docs/architecture.drawio.svg)

## Installation

You can install the latest `surveilr` using either of the following one-liners:

```bash
$ curl -sL https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/install.sh | sh

# if you want a different install path
$ SURVEILR_HOME="$HOME/bin" curl -sL https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/install.sh | sh
```

If you use `eget`:

```bash
$ eget opsfolio/resource-surveillance --asset tar.gz
```

## Usage

Unless you set the `SURVEILR_STATEDB_FS_PATH` env var, the default _Resource
Surveillance State ("surveillance state") SQLite Database_ will be
`resource-surveillance.sqlite.db`.

If you have many surveillance state SQLite databases and you want to query them
as one database instead of individual `RSSD`s, you should use a pattern like
like this so they can be easily merged using `surveilr admin merge-sql`. Add any
kind of _identifying_ or _differenting_ field(s) like `$(hostname)` into the
`RSSD` filename.

```bash
$ export SURVEILR_STATEDB_FS_PATH="resource-surveillance-$(hostname).sqlite.db"
```

Here's how you use the most common command patterns:

```bash
$ surveilr --help                         # get CLI help (pay special attention to ENV var names)
$ surveilr fs-walk                        # walk the current working directory (CWD)
$ surveilr fs-walk -r /other -r /other2   # walk some other director(ies)
$ surveilr fs-walk --stats                # walk the current working directory (CWD) show stats afterwards
$ surveilr --completions fish | source    # setup completions to reduce typing
```

Merging multiple _Resource Surveillance State SQLite Databases_into one using
`surveilr` _without_ running `sqlite3`:

```bash
$ surveilr admin merge                         # execute merge SQL for all files in the current path
$ surveilr admin merge --candidates "**/*.db"  # execute merge SQL for specific globs in the current path
```

The `surveilr admin merge` above prepares the SQL to merge multiple databases
into one and executes it automatically creating
`resource-surveillance-aggregated.sqlite.db` (you can override the name using
`-d`).

Generating SQL to merge multiple _Resource Surveillance State SQLite Databases_
into one, inspecting it, and then executing _using_ `sqlite3`:

```bash
$ surveilr admin merge --sql-only
$ surveilr admin merge --candidates "**/*.db" --sql-only
$ surveilr admin merge --candidates "**/*.db" -i "x*.db" --sql-only # -i ignores certain candidates
$ surveilr admin merge --sql-only > merge.sql
```

Merging multiple databases into one using generated SQL (using `sqlite3` shell)
after generating the code:

```bash
$ surveilr admin init -d target.sqlite.db -r \
  && surveilr admin merge -d target.sqlite.db --sql-only \
   | sqlite3 target.sqlite.db
```

The CLI multi-command pipe above does three things:

1. `surveilr admin init` initializes an empty `target.sqlite.db` (`-r` removes
   it if it exists)
2. `surveilr admin merge --sql-only` generates the merge SQL for all databases
   except `target.sqlite.db`
3. `sqlite3` pipe at the end just executes the generated SQL using SQLite 3
   shell and produces merged `target.sqlite.db`

Once `target.sqlite.db` is created after step 3, none of the original
device-specific `RSSD`s are required and `target.sqlite.db` is independent of
`surveilr` as well.

Notebook use cases:

```bash
$ surveilr notebooks ls                                     # list all notebooks and cells available, with migrations status
$ surveilr notebooks cat --cell infoSchemaOsQueryATCs       # export the information schema as osQuery ATC
$ surveilr notebooks cat --cell notebooksInfoSchemaDiagram  # show the notebooks admin PlanUML ERD stored in the database
$ surveilr notebooks cat --cell surveilrInfoSchemaDiagram   # show the surveilr PlanUML ERD stored in the database
```

In order to ensure that the Resource Surveillance agent is extensible, we
leverage SQLite heavily for both storage of data but also storing the SQL it
needs to bootstrap itself, perform migrations, and conduct regular
administrative and query operations.

The key to that extensibility is the `code_notebook_cell` table which stores SQL
(called _SQL notebook cells_) or other interpretable code in the database so
that once the database is created, all SQL and related code is part of the
database and may be executed like this from the CLI using any environment that
supports SQLite shell or SQLite drivers:

```bash
$ sqlite3 xyz.db "select sql from code_notebook_cell where code_notebook_cell_id = 'infoSchemaMarkdown'" | sqlite3 xyz.db
```

You can pass in arguments using `.parameter` or `sql_parameters` table, like:

```bash
$ echo ".parameter set X Y; $(sqlite3 xyz.db \"SELECT sql FROM code_notebook_cell where code_notebook_cell_id = 'init'\")" | sqlite3 xyz.db
```

Anywhere you see `surveilr notebooks cat` those can be run directly through
SQLite, the following two commands do the same thing:

```bash
$ surveilr notebooks cat --cell infoSchemaOsQueryATCs | sqlite3 resource-surveillance.sqlite.db
$ sqlite3 resource-surveillance.sqlite.db "select interpretable_code from stored_notebook_cell where cell_name = 'infoSchemaOsQueryATCs'" | sqlite3 device-content.sqlite.db
```

See [CLI Help](support/docs/CLI-help.md) for summary of what `surveilr --help`
provides. Though [CLI Help](support/docs/CLI-help.md) is a good reference, it's
best to depend on `surveilr --help` and `surveilr <command> --help` because it
will more accurate for the latest version.

## Database Documentation

- [SQLite State Schema Documentation](support/docs/surveilr-state-schema/README.md)
- [SQLite Notebooks Schema Documentation](support/docs/surveilr-code-notebooks-schema/README.md)

To generate schema docs:

```bash
$ just tbls
```

### AI Prompts

In order to make it easier to understand how to generate `surveilr` SQL, you can
use these prompts stored in the notebooks:

```bash
$ surveilr notebooks cat --cell "%understand service schema%"
$ surveilr notebooks cat --cell "%understand notebooks schema%"
```

The output of the first one is a good way to help ChatGPT or other LLM to
understand the `surveilr` service SQL schema (`device`, `uniform_resource`,
`fs-walk`, etc) and ask it questions to generate SQL for you. The second one is
a good way to help ChatGPT or other LLM to understand the `surveilr` notebooks
schema and ask it questions to generate SQL specifically for the _notebooks_
capability.

## Development

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
