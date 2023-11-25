# Command-Line Help for `surveilr`

This document contains the help content for the `surveilr` command-line program.

**Command Overview:**

* [`surveilr`↴](#surveilr)
* [`surveilr admin`↴](#surveilr-admin)
* [`surveilr admin init`↴](#surveilr-admin-init)
* [`surveilr admin merge`↴](#surveilr-admin-merge)
* [`surveilr admin cli-help-md`↴](#surveilr-admin-cli-help-md)
* [`surveilr capturable-exec`↴](#surveilr-capturable-exec)
* [`surveilr capturable-exec ls`↴](#surveilr-capturable-exec-ls)
* [`surveilr capturable-exec test`↴](#surveilr-capturable-exec-test)
* [`surveilr ingest`↴](#surveilr-ingest)
* [`surveilr notebooks`↴](#surveilr-notebooks)
* [`surveilr notebooks cat`↴](#surveilr-notebooks-cat)
* [`surveilr notebooks ls`↴](#surveilr-notebooks-ls)
* [`surveilr shell`↴](#surveilr-shell)
* [`surveilr shell json`↴](#surveilr-shell-json)

## `surveilr`

**Usage:** `surveilr [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `admin` — Admin / maintenance utilities
* `capturable-exec` — Capturable Executables (CE) maintenance tools
* `ingest` — Ingest content from device file system and other sources
* `notebooks` — Notebooks maintenance utilities
* `shell` — Deno Task Shell utilities

###### **Options:**

* `--device-name <DEVICE_NAME>` — How to identify this device

  Default value: `Titan`
* `-d`, `--debug` — Turn debugging information on (repeat for higher levels)



## `surveilr admin`

Admin / maintenance utilities

**Usage:** `surveilr admin <COMMAND>`

###### **Subcommands:**

* `init` — initialize an empty database with bootstrap.sql
* `merge` — merge multiple surveillance state databases into a single one
* `cli-help-md` — generate CLI help markdown



## `surveilr admin init`

initialize an empty database with bootstrap.sql

**Usage:** `surveilr admin init [OPTIONS]`

###### **Options:**

* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `resource-surveillance.sqlite.db`
* `-I`, `--state-db-init-sql <STATE_DB_INIT_SQL>` — one or more globs to match as SQL files and batch execute them in alpha order
* `-r`, `--remove-existing-first` — remove the existing database first
* `--with-device` — add the current device in the empty database's device table



## `surveilr admin merge`

merge multiple surveillance state databases into a single one

**Usage:** `surveilr admin merge [OPTIONS]`

###### **Options:**

* `-c`, `--candidates <CANDIDATES>` — one or more DB name globs to match and merge

  Default value: `*.db`
* `-i`, `--ignore-candidates <IGNORE_CANDIDATES>` — one or more DB name globs to ignore if they match
* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database with merged content

  Default value: `resource-surveillance-aggregated.sqlite.db`
* `-I`, `--state-db-init-sql <STATE_DB_INIT_SQL>` — one or more globs to match as SQL files and batch execute them in alpha order
* `-r`, `--remove-existing-first` — remove the existing database first
* `--sql-only` — only generate SQL and emit to STDOUT (no actual merge)



## `surveilr admin cli-help-md`

generate CLI help markdown

**Usage:** `surveilr admin cli-help-md`



## `surveilr capturable-exec`

Capturable Executables (CE) maintenance tools

**Usage:** `surveilr capturable-exec <COMMAND>`

###### **Subcommands:**

* `ls` — list potential capturable executables
* `test` — test capturable executables



## `surveilr capturable-exec ls`

list potential capturable executables

**Usage:** `surveilr capturable-exec ls [OPTIONS]`

###### **Options:**

* `-r`, `--root-fs-path <ROOT_FS_PATH>` — one or more root paths to ingest

  Default value: `.`
* `-i`, `--ignore-fs-entry <IGNORE_FS_ENTRY>` — reg-exes to use to ignore files in root-path(s)

  Default value: `/(\\.git|node_modules)/`
* `--capture-fs-exec <CAPTURE_FS_EXEC>` — reg-exes to use to execute and capture STDOUT, STDERR (e.g. *.surveilr[json].sh) with "nature" capture group

  Default value: `surveilr\[(?P<nature>[^\]]*)\]`
* `--captured-fs-exec-sql <CAPTURED_FS_EXEC_SQL>` — reg-exes that will signify which captured executables' output should be treated as batch SQL

  Default value: `surveilr-SQL`
* `--markdown` — emit the results as markdown, not a simple table



## `surveilr capturable-exec test`

test capturable executables

**Usage:** `surveilr capturable-exec test [OPTIONS] --fs-path <FS_PATH>`

###### **Options:**

* `-f`, `--fs-path <FS_PATH>`
* `--capture-fs-exec <CAPTURE_FS_EXEC>` — reg-exes to use to execute and capture STDOUT, STDERR (e.g. *.surveilr[json].sh) with "nature" capture group

  Default value: `surveilr\[(?P<nature>[^\]]*)\]`
* `--captured-fs-exec-sql <CAPTURED_FS_EXEC_SQL>` — reg-exes that will signify which captured executables' output should be treated as batch SQL

  Default value: `surveilr-SQL`



## `surveilr ingest`

Ingest content from device file system and other sources

**Usage:** `surveilr ingest [OPTIONS]`

###### **Options:**

* `-b`, `--behavior <BEHAVIOR>` — the behavior name in `behavior` table
* `-r`, `--root-fs-path <ROOT_FS_PATH>` — one or more root paths to ingest

  Default value: `.`
* `-i`, `--ignore-fs-entry <IGNORE_FS_ENTRY>` — reg-exes to use to ignore files in root-path(s)

  Default value: `/(\\.git|node_modules)/`
* `--compute-fs-content-digests <COMPUTE_FS_CONTENT_DIGESTS>` — reg-exes to use to compute digests for

  Default value: `.*`
* `--surveil-fs-content <SURVEIL_FS_CONTENT>` — reg-exes to use to load content for entry instead of just walking

  Default values: `\.(md|mdx|html|json|jsonc|tap|txt|text|toml|yaml)$`, `surveilr\[(?P<nature>[^\]]*)\]`
* `--capture-fs-exec <CAPTURE_FS_EXEC>` — reg-exes to use to execute and capture STDOUT, STDERR (e.g. *.surveilr[json].sh) with "nature" capture group

  Default value: `surveilr\[(?P<nature>[^\]]*)\]`
* `--captured-fs-exec-sql <CAPTURED_FS_EXEC_SQL>` — reg-exes that will signify which captured executables' output should be treated as batch SQL

  Default value: `surveilr-SQL`
* `-n`, `--notebook <NOTEBOOK>` — search cells in these notebooks (include % for LIKE otherwise =)
* `-c`, `--cell <CELL>` — use these cells' content as ingestion code (include % for LIKE otherwise =)
* `-N`, `--nature-bind <NATURE_BIND>` — bind an unknown nature (file extension), the key, to a known nature the value "text=text/plain,yaml=application/yaml"
* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `resource-surveillance.sqlite.db`
* `-I`, `--state-db-init-sql <STATE_DB_INIT_SQL>` — one or more globs to match as SQL files and batch execute them in alpha order
* `--include-state-db-in-ingestion` — include the surveil database in the ingestion candidates
* `--stats` — show stats as an ASCII table after completion
* `--stats-json` — show stats in JSON after completion
* `--save-behavior <SAVE_BEHAVIOR>` — save the options as a new behavior



## `surveilr notebooks`

Notebooks maintenance utilities

**Usage:** `surveilr notebooks [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `cat` — Notebooks' cells emit utilities
* `ls` — list all notebooks

###### **Options:**

* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `resource-surveillance.sqlite.db`
* `-I`, `--state-db-init-sql <STATE_DB_INIT_SQL>` — one or more globs to match as SQL files and batch execute them in alpha order



## `surveilr notebooks cat`

Notebooks' cells emit utilities

**Usage:** `surveilr notebooks cat [OPTIONS]`

###### **Options:**

* `-n`, `--notebook <NOTEBOOK>` — search for these notebooks (include % for LIKE otherwise =)
* `-c`, `--cell <CELL>` — search for these cells (include % for LIKE otherwise =)
* `-s`, `--seps` — add separators before each cell



## `surveilr notebooks ls`

list all notebooks

**Usage:** `surveilr notebooks ls [OPTIONS]`

###### **Options:**

* `-m`, `--migratable` — list all SQL cells that will be handled by execute_migrations



## `surveilr shell`

Deno Task Shell utilities

**Usage:** `surveilr shell <COMMAND>`

###### **Subcommands:**

* `json` — Execute a command string in [Deno Task Shell](https://docs.deno.com/runtime/manual/tools/task_runner) returns JSON



## `surveilr shell json`

Execute a command string in [Deno Task Shell](https://docs.deno.com/runtime/manual/tools/task_runner) returns JSON

**Usage:** `surveilr shell json [OPTIONS] --command <COMMAND>`

###### **Options:**

* `-c`, `--command <COMMAND>` — the command that would work as a Deno Task
* `--cwd <CWD>` — use this as the current working directory (CWD)
* `-s`, `--stdout-only` — emit stdout only, without the exec status code and stderr

  Default value: `false`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

