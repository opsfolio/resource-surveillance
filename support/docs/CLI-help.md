# Command-Line Help for `surveilr`

This document contains the help content for the `surveilr` command-line program.

**Command Overview:**

* [`surveilr`↴](#surveilr)
* [`surveilr admin`↴](#surveilr-admin)
* [`surveilr admin init`↴](#surveilr-admin-init)
* [`surveilr admin merge`↴](#surveilr-admin-merge)
* [`surveilr admin cli-help-md`↴](#surveilr-admin-cli-help-md)
* [`surveilr admin test`↴](#surveilr-admin-test)
* [`surveilr admin test classifiers`↴](#surveilr-admin-test-classifiers)
* [`surveilr capturable-exec`↴](#surveilr-capturable-exec)
* [`surveilr capturable-exec ls`↴](#surveilr-capturable-exec-ls)
* [`surveilr capturable-exec test`↴](#surveilr-capturable-exec-test)
* [`surveilr capturable-exec test file`↴](#surveilr-capturable-exec-test-file)
* [`surveilr capturable-exec test task`↴](#surveilr-capturable-exec-test-task)
* [`surveilr ingest`↴](#surveilr-ingest)
* [`surveilr ingest files`↴](#surveilr-ingest-files)
* [`surveilr ingest tasks`↴](#surveilr-ingest-tasks)
* [`surveilr notebooks`↴](#surveilr-notebooks)
* [`surveilr notebooks cat`↴](#surveilr-notebooks-cat)
* [`surveilr notebooks ls`↴](#surveilr-notebooks-ls)

## `surveilr`

**Usage:** `surveilr [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `admin` — Admin / maintenance utilities
* `capturable-exec` — Capturable Executables (CE) maintenance tools
* `ingest` — Ingest content from device file system and other sources
* `notebooks` — Notebooks maintenance utilities

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
* `test` — generate CLI help markdown



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



## `surveilr admin test`

generate CLI help markdown

**Usage:** `surveilr admin test <COMMAND>`

###### **Subcommands:**

* `classifiers` — test capturable executables files



## `surveilr admin test classifiers`

test capturable executables files

**Usage:** `surveilr admin test classifiers [OPTIONS]`

###### **Options:**

* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `resource-surveillance.sqlite.db`
* `-I`, `--state-db-init-sql <STATE_DB_INIT_SQL>` — one or more globs to match as SQL files and batch execute them in alpha order
* `--builtins` — only show the builtins, not from the database



## `surveilr capturable-exec`

Capturable Executables (CE) maintenance tools

**Usage:** `surveilr capturable-exec <COMMAND>`

###### **Subcommands:**

* `ls` — list potential capturable executables
* `test` — test capturable executables files



## `surveilr capturable-exec ls`

list potential capturable executables

**Usage:** `surveilr capturable-exec ls [OPTIONS]`

###### **Options:**

* `-r`, `--root-fs-path <ROOT_FS_PATH>` — one or more root paths to ingest

  Default value: `.`
* `--markdown` — emit the results as markdown, not a simple table



## `surveilr capturable-exec test`

test capturable executables files

**Usage:** `surveilr capturable-exec test <COMMAND>`

###### **Subcommands:**

* `file` — test capturable executables files
* `task` — Execute a task string as if it was run by `ingest tasks` and show the output



## `surveilr capturable-exec test file`

test capturable executables files

**Usage:** `surveilr capturable-exec test file --fs-path <FS_PATH>`

###### **Options:**

* `-f`, `--fs-path <FS_PATH>`



## `surveilr capturable-exec test task`

Execute a task string as if it was run by `ingest tasks` and show the output

**Usage:** `surveilr capturable-exec test task [OPTIONS]`

###### **Options:**

* `-s`, `--stdin` — send commands in via STDIN the same as with `ingest tasks` and just emit the output
* `-t`, `--task <TASK>` — one or more commands that would work as a Deno Task line
* `--cwd <CWD>` — use this as the current working directory (CWD)



## `surveilr ingest`

Ingest content from device file system and other sources

**Usage:** `surveilr ingest <COMMAND>`

###### **Subcommands:**

* `files` — Ingest content from device file system and other sources
* `tasks` — Notebooks maintenance utilities



## `surveilr ingest files`

Ingest content from device file system and other sources

**Usage:** `surveilr ingest files [OPTIONS]`

###### **Options:**

* `--dry-run` — don't run the ingestion, just report statistics
* `-b`, `--behavior <BEHAVIOR>` — the behavior name in `behavior` table
* `-r`, `--root-fs-path <ROOT_FS_PATH>` — one or more root paths to ingest

  Default value: `.`
* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `resource-surveillance.sqlite.db`
* `-I`, `--state-db-init-sql <STATE_DB_INIT_SQL>` — one or more globs to match as SQL files and batch execute them in alpha order
* `--include-state-db-in-ingestion` — include the surveil database in the ingestion candidates
* `--stats` — show stats as an ASCII table after completion
* `--stats-json` — show stats in JSON after completion
* `--save-behavior <SAVE_BEHAVIOR>` — save the options as a new behavior



## `surveilr ingest tasks`

Notebooks maintenance utilities

**Usage:** `surveilr ingest tasks [OPTIONS]`

###### **Options:**

* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `resource-surveillance.sqlite.db`
* `-I`, `--state-db-init-sql <STATE_DB_INIT_SQL>` — one or more globs to match as SQL files and batch execute them in alpha order
* `--stdin` — read tasks from STDIN
* `--stats` — show session stats after completion
* `--stats-json` — show session stats as JSON after completion



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



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

