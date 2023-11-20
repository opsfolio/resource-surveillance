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
* [`surveilr notebooks`↴](#surveilr-notebooks)
* [`surveilr notebooks cat`↴](#surveilr-notebooks-cat)
* [`surveilr notebooks ls`↴](#surveilr-notebooks-ls)
* [`surveilr fs-walk`↴](#surveilr-fs-walk)

## `surveilr`

**Usage:** `surveilr [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `admin` — Admin / maintenance utilities
* `capturable-exec` — Capturable Executables (CE) maintenance tools
* `notebooks` — Notebooks maintenance utilities
* `fs-walk` — Walks the device file system

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



## `surveilr capturable-exec ls`

list potential capturable executables

**Usage:** `surveilr capturable-exec ls [OPTIONS]`

###### **Options:**

* `-r`, `--root-path <ROOT_PATH>` — one or more root paths to walk

  Default value: `.`
* `-i`, `--ignore-entry <IGNORE_ENTRY>` — reg-exes to use to ignore files in root-path(s)

  Default value: `/(\\.git|node_modules)/`
* `--capture-exec <CAPTURE_EXEC>` — reg-exes to use to execute and capture STDOUT, STDERR (e.g. *.surveilr[json].sh) with "nature" capture group

  Default value: `surveilr\[(?P<nature>[^\]]*)\]`
* `--captured-exec-sql <CAPTURED_EXEC_SQL>` — reg-exes that will signify which captured executables' output should be treated as batch SQL

  Default value: `surveilr-SQL`
* `--markdown` — emit the results as markdown, not a simple table



## `surveilr notebooks`

Notebooks maintenance utilities

**Usage:** `surveilr notebooks [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `cat` — Notebooks' cells emit utilities
* `ls` — list all notebooks

###### **Options:**

* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `resource-surveillance.sqlite.db`



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



## `surveilr fs-walk`

Walks the device file system

**Usage:** `surveilr fs-walk [OPTIONS]`

###### **Options:**

* `-b`, `--behavior <BEHAVIOR>` — the behavior name in `behavior` table
* `-r`, `--root-path <ROOT_PATH>` — one or more root paths to walk

  Default value: `.`
* `-i`, `--ignore-entry <IGNORE_ENTRY>` — reg-exes to use to ignore files in root-path(s)

  Default value: `/(\\.git|node_modules)/`
* `--compute-digests <COMPUTE_DIGESTS>` — reg-exes to use to compute digests for

  Default value: `.*`
* `--surveil-content <SURVEIL_CONTENT>` — reg-exes to use to load content for entry instead of just walking

  Default values: `\.(md|mdx|html|json|jsonc|tap|txt|text|toml|yaml)$`, `surveilr\[(?P<nature>[^\]]*)\]`
* `--capture-exec <CAPTURE_EXEC>` — reg-exes to use to execute and capture STDOUT, STDERR (e.g. *.surveilr[json].sh) with "nature" capture group

  Default value: `surveilr\[(?P<nature>[^\]]*)\]`
* `--captured-exec-sql <CAPTURED_EXEC_SQL>` — reg-exes that will signify which captured executables' output should be treated as batch SQL

  Default value: `surveilr-SQL`
* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `resource-surveillance.sqlite.db`
* `--include-state-db-in-walk` — include the surveil database in the walk
* `--stats` — show stats as an ASCII table after completion
* `--stats-json` — show stats in JSON after completion
* `--save-behavior <SAVE_BEHAVIOR>` — save the options as a new behavior



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

