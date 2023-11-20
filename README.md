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

Here's how you get help for the various commands that `surveilr` provides:

```bash
$ surveilr --version                      # version information
$ surveilr --help                         # get CLI help (pay special attention to ENV var names)
$ surveilr --completions fish | source    # setup shell completions to reduce typing
```

See [CLI Help](support/docs/CLI-help.md) for summary of what `surveilr --help`
provides. Though [CLI Help](support/docs/CLI-help.md) is a good reference, it's
best to depend on `surveilr --help` and `surveilr <command> --help` because it
will more accurate for the latest version.

## Creating `RSSD`s by walking the file system (`fs-walk`)

Unless you set the `SURVEILR_STATEDB_FS_PATH` env var, the default _Resource
Surveillance State Database_ ("RSSD") will be `resource-surveillance.sqlite.db`.

If you have many surveillance state SQLite databases and you want to query them
as one database instead of individual `RSSD`s, you should use a pattern like
like this so they can be easily merged using `surveilr admin merge-sql`. Add any
kind of _identifying_ or _differenting_ field(s) like `$(hostname)` into the
`RSSD` filename.

```bash
$ export SURVEILR_STATEDB_FS_PATH="resource-surveillance-$(hostname).sqlite.db"
```

Here's how you use the most common `fs-walk`patterns:

```base
$ surveilr fs-walk --help                 # explain the `fs-walk` subcommand
$ surveilr fs-walk                        # walk the current working directory (CWD)
$ surveilr fs-walk -r /other -r /other2   # walk some other director(ies)
$ surveilr fs-walk --stats                # walk the current working directory (CWD) show stats afterwards
```

## Merging multiple `RSSD`s into one using `surveilr` (`admin merge`)

Merging multiple _Resource Surveillance State SQLite Databases_ into one using
`surveilr` _without_ running external `sqlite3`:

```bash
$ surveilr admin merge --help                  # explain the `admin merge` subcommand
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

### Merging multiple `RSSD`s into one using `sqlite3` (`admin merge --sql-only`)

Because merging multiple database can sometimes fail due to unforseen data
issues, you can use `sqlite3` to merge multiple databases using
`surveilr`-generated SQL after inspecting it. Here's how:

```bash
$ surveilr admin init -d target.sqlite.db -r \
  && surveilr admin merge -d target.sqlite.db --sql-only \
   | sqlite3 target.sqlite.db
```

The CLI multi-command pipe above does three things:

1. `surveilr admin init` initializes an empty `target.sqlite.db` (`-r` removes
   it if it exists)
2. `surveilr admin merge --sql-only` generates the merge SQL for all databases
   except `target.sqlite.db`; to inspect the SQL you can save it to a file
   `surveilr admin merge -d target.sqlite.db --sql-only > merge.sql`.
3. `sqlite3` pipe at the end just executes the generated SQL using SQLite 3
   shell and produces merged `target.sqlite.db`

Once `target.sqlite.db` is created after step 3, none of the original
device-specific `RSSD`s are required and `target.sqlite.db` is independent of
`surveilr` as well.

## Files as Resources vs. Capturable Executables as Resources

When `fs-walk` command runs, it's main job is to find files and store them in
`uniform_resource` table as records. If the content of a file is already stored
in the file system this works well. However, sometimes we need to generate the
content of a file (or group of files) and store the output of the generated
files. That's where the idea of _Capturable Executables_ (`CEs`) comes in.

CEs allow you to pass in arguments or behaviors to the `fs-walk` command that
allows certain patterns of files to be executed in a safe shell, and their
STDOUT and STDERR captured and stored in `uniform_resource`. These scripts are
referred to as _capturable executables_ or `CE`s and are influenced through
_Processing Instructions_ (`PI`s) in file names.

For example, if we want `fs-walk`, as it encounters a `abc.surveilr.sh` file to
execute it, we can pass in `--capture_exec "surverilr[json]"` and it will
execute the script, treat the output as JSON, and store it in
`uniform_resource`. `surverilr[json]` becomes what is known as a `CE` Resource
Surveillance _Processing Instruction_ (`PI`) and the pattern is arbitrary so
long as the _nature_ is a named Rust reg ex capture group liked
`surveilr\[(?P<nature>[^]]*)\]` (focus on `nature`, you can test this regular
expressions at https://regex101.com/r/sVroiN/1).

This _Capturable Executables_ functionality is available:

- Calls an executable without any parameters and assumes the output is whatever
  is in `[xyz]` PI as the `uniform_resource` _nature_.
- If the filename is something like `myfile.surveilr-SQL.sh` it means that the
  output of the command will be treated as batch SQL and executed on the target
  SQLite database in the same transaction as the primary database. If we pass in
  everything through STDIN (see below), `INSERT INTO` should be easy.
- Need to capture status code from the executable and STDERR from subprocess and
  pass it back so it can be stored in
  `walk_session_path_fs_entry`.`captured_executable` JSON column along with
  `args` and `stdin`.
- Pass in the device, behavior, and other context information through CLI
  parameters or STDIN to the shell script. The input (STDIN) should look like
  this and contain a reasonably complete context so that executables know how to
  generate their output:

```json
{
  "surveilr-fs-walk": {
    "args": {
      "state_db_fs_path": "./e2e-test-state.sqlite.db"
    },
    "behavior": {
      "capturable_executables": [
        "surveilr\\[(?P<nature>[^\\]]*)\\]"
      ],
      "compute_digests": [".*"],
      "ignore_regexs": [
        "/(\\.git|node_modules)/"
      ],
      "ingest_content": [
        "\\.(md|mdx|html|json|jsonc|toml|yaml)$",
        "surveilr\\[(?P<nature>[^\\]]*)\\]"
      ],
      "root_paths": ["./support/test-fixtures"]
    },
    "device": { "device_id": "01HFHZGEZC763PWRBV2WKXBJH0" },
    "env": {
      "current_dir": "/home/snshah/workspaces/github.com/opsfolio/resource-surveillance"
    },
    "session": {
      "entry": {
        "path": "/home/snshah/workspaces/github.com/opsfolio/resource-surveillance/support/test-fixtures/echo-stdin.surveilr[json].sh"
      },
      "walk-path-id": "01HFHZGEZEDTW29BWWSEDE46WH",
      "walk-session-id": "01HFHZGEZD31S0V1EYBW4TT530"
    }
  }
}
```

### Testing Capturable Executables

Try to keep CEs individually testable as independent scripts. You can validate
capturable executables by using `surveilr cap-exec` subcommands.

```bash
$ surveilr cap-exec --help                       # see all the options
$ surveilr cap-exec ls                           # scan for CEs and show a table of what's found
$ surveilr cap-exec ls --markdown > cap-exec.md  # find CEs, try to execute them, store their output in a Markdown
```

Running `cap-exec ls` should show something similar to this:

```
| Executable                                                                     | Nature                        | Issue             |
|:------------------------------------------------------------------------------:|:-----------------------------:|:-----------------:|
| ./support/test-fixtures/idempotent.surveilr-SQL.sh                             | batched SQL                   |                   |
| ./support/test-fixtures/capturable-executable.surveilr[json].sh                | json                          |                   |
| ./support/test-fixtures/capturable-executable.surveilr[json].ts                | json                          |                   |
| ./support/test-fixtures/echo-stdin.surveilr[json].sh                           | json                          |                   |
| ./support/test-fixtures/capturable-executable-bad-exit-status.surveilr[txt].sh | txt                           |                   |
| ./support/test-fixtures/capturable-executable-no-permissions.surveilr[json].sh | Executable Permission Not Set | chmod +x required |
```

Running `cap-exec ls --markdown` generates a Markdown document that you can use
to learn more about what `STDIN`, `STDOUT`, and `STDERR` streams will be created
during `fs-walk`.

### Capturable Executables Examples

See these examples in `support/test-fixtures`:

- `capturable-executable.surveilr[json].ts` shows how to use simple JSON output
  from a Deno script and store it in `uniform_resource`
- `capturable-executable.surveilr[json].sh` shows how to use simple JSON output
  from a Bash script and store it in `uniform_resource`
- `echo-stdin.surveilr[json].sh` shows how to accept STDIN and emit JSON as
  STDOUT -- this allows more complex processing by getting additional context
  from surveilr and doing something special in a script.
- `idempotent.surveilr-SQL.sh` shows how a script can generate SQL and
  `surveilr` will execute the SQL as a batch in the same transaction (WARNING:
  SQL is unsanitized and this might be a security hole so be careful turning it
  on).

How to control the behavior of _Capturable Executables_ filenames:

- `surveilr fs-walk --captured-exec-sql` Regexp(s) control which files are
  considered capturable executables who output will be captured and executed as
  "SQL batch"
- `surveilr fs-walk --capture-exec` Regexp(s) control which files are considered
  capturable executables

Full diagnostics of STDIN, STDOUT, STDERR, etc. are present in the
`ur_session_path_fs_entry` row for all scripts as they're encountered. If you
need more features, submit tickets.

## Code Notebooks

In order to ensure that the Resource Surveillance agent is extensible, we
leverage SQLite heavily for both storage of data but also storing the SQL it
needs to bootstrap itself, perform migrations, and conduct regular
administrative and query operations.

```bash
$ surveilr notebooks ls                                     # list all notebooks and cells available, with migrations status
$ surveilr notebooks cat --cell infoSchemaOsQueryATCs       # export the information schema as osQuery ATC
$ surveilr notebooks cat --cell notebooksInfoSchemaDiagram  # show the notebooks admin PlanUML ERD stored in the database
$ surveilr notebooks cat --cell surveilrInfoSchemaDiagram   # show the surveilr PlanUML ERD stored in the database
```

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
