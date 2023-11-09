# Command-Line Help for `surveilr`

This document contains the help content for the `surveilr` command-line program.

**Command Overview:**

* [`surveilr`↴](#surveilr)
* [`surveilr admin`↴](#surveilr-admin)
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

* `cli-help-md` — generate CLI help markdown



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

* `--surveil-db-fs-path <SURVEIL_DB_FS_PATH>` — target SQLite database

  Default value: `./resource-surveillance.sqlite.db`



## `surveilr notebooks cat`

Notebooks' cells emit utilities

**Usage:** `surveilr notebooks cat [OPTIONS]`

###### **Options:**

* `-n`, `--notebook <NOTEBOOK>`
* `-c`, `--cell <CELL>`



## `surveilr notebooks ls`

list all notebooks

**Usage:** `surveilr notebooks ls`



## `surveilr fs-walk`

Walks the device file system

**Usage:** `surveilr fs-walk [OPTIONS]`

###### **Options:**

* `-r`, `--root-path <ROOT_PATH>` — one or more root paths to walk

  Default value: `.`
* `-i`, `--ignore-entry <IGNORE_ENTRY>` — reg-exes to use to ignore files in root-path(s)

  Default value: `/(\.git|node_modules)/`
* `--compute-digests <COMPUTE_DIGESTS>` — reg-exes to use to compute digests for

  Default value: `.*`
* `--surveil-content <SURVEIL_CONTENT>` — reg-exes to use to load content for entry instead of just walking

  Default value: `\.(md|mdx|html|json|jsonc)$`
* `--surveil-db-fs-path <SURVEIL_DB_FS_PATH>` — target SQLite database

  Default value: `./resource-surveillance.sqlite.db`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

