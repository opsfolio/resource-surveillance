# Command-Line Help for `surveilr`

This document contains the help content for the `surveilr` command-line program.

**Command Overview:**

* [`surveilr`↴](#surveilr)
* [`surveilr admin`↴](#surveilr-admin)
* [`surveilr admin init`↴](#surveilr-admin-init)
* [`surveilr admin merge-sql`↴](#surveilr-admin-merge-sql)
* [`surveilr admin cli-help-md`↴](#surveilr-admin-cli-help-md)
* [`surveilr notebooks`↴](#surveilr-notebooks)
* [`surveilr notebooks cat`↴](#surveilr-notebooks-cat)
* [`surveilr notebooks ls`↴](#surveilr-notebooks-ls)
* [`surveilr fs-walk`↴](#surveilr-fs-walk)

## `surveilr`

**Usage:** `surveilr [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `admin` — Admin / maintenance utilities
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
* `merge-sql` — generate SQLite SQL that will merge multiple databases into a single one
* `cli-help-md` — generate CLI help markdown



## `surveilr admin init`

initialize an empty database with bootstrap.sql

**Usage:** `surveilr admin init [OPTIONS]`

###### **Options:**

* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `./resource-surveillance.sqlite.db`
* `-r`, `--remove-existing-first` — remove the existing database first



## `surveilr admin merge-sql`

generate SQLite SQL that will merge multiple databases into a single one

**Usage:** `surveilr admin merge-sql [OPTIONS]`

###### **Options:**

* `-d`, `--db-glob <DB_GLOB>` — one or more DB name globs to match and merge

  Default value: `*.db`
* `-i`, `--db-glob-ignore <DB_GLOB_IGNORE>` — one or more DB name globs to ignore if they match



## `surveilr admin cli-help-md`

generate CLI help markdown

**Usage:** `surveilr admin cli-help-md`



## `surveilr notebooks`

Notebooks maintenance utilities

**Usage:** `surveilr notebooks [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `cat` — Notebooks' cells emit utilities
* `ls` — list all notebooks

###### **Options:**

* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `./resource-surveillance.sqlite.db`



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

  Default value: `/(\.git|node_modules)/`
* `--compute-digests <COMPUTE_DIGESTS>` — reg-exes to use to compute digests for

  Default value: `.*`
* `--surveil-content <SURVEIL_CONTENT>` — reg-exes to use to load content for entry instead of just walking

  Default value: `\.(md|mdx|html|json|jsonc|toml|yaml)$`
* `-d`, `--state-db-fs-path <STATE_DB_FS_PATH>` — target SQLite database

  Default value: `./resource-surveillance.sqlite.db`
* `--include-state-db-in-walk` — include the surveil database in the walk
* `--stats` — show stats as an ASCII table after completion
* `--stats-json` — show stats in JSON after completion
* `--save-behavior <SAVE_BEHAVIOR>` — save the options as a new behavior



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

