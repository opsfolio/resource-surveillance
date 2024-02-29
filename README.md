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

## Checking what can be "walked" in the file system

Before you do any ingestion into SQLite `RSSD`s, you can get some statistics on
what will be ingested when you use the `ingest` command in the next section. Use
`--dry-run` to see a summary of what files will be ingested.

```bash
$ surveilr ingest files --dry-run                        # show stats for CWD
$ surveilr ingest files --dry-run -r /other -r /other2   # test some other director(ies)
```

## Creating `RSSD`s by "walking" the file system

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

Here's how you use the most common `ingest` patterns:

```bash
$ surveilr ingest files --help                 # explain the `ingest` subcommand
$ surveilr ingest files                        # walk the current working directory (CWD)
$ surveilr ingest files -r /other -r /other2   # walk some other director(ies)
$ surveilr ingest files --stats                # walk the current working directory (CWD) show stats afterwards
```

## Creating `RSSD`s by executing shell tasks

The `surveilr ingest tasks` commands accepts one or more lines of Deno Task
Shell commands / tasks through STDIN, executes them one by one, and inserts each
output as JSON into `uniform_resource`.

Each line in STDIN can be one of the following:

- **Simple text**. A line of text that is not in `JSONL` format will be treated
  as an "anonymous" (unidentified) command string, executed with the assumption
  that the command output is in JSON, and stored in `uniform_resource`.
- **JSONL**. A line of text that is in `JSONL` format will be treated as an JSON
  object of the type `{ key: string, nature?: string }`, parsed, the value of
  `key` is executed, and stored in `uniform_resource` using `key` is the
  identifier with an optional `nature`. If `nature` is not supplied, the output
  of the command is assumed to be JSON.

Examples:

```bash
# single command without identifier or nature, surveilr expects JSON
$ echo "osqueryi \"select * from users\" --json" | surveilr ingest tasks

# single command with identifier and nature
$ echo "{ \"my-osquery-test\": \"osqueryi 'select * from users'\", \"nature\": \"txt\" }" | surveilr ingest tasks

# multiple commands whether each line can be a JSONL formatted object;
# the following runs Deno to grab a locak package.json file, extract all scripts
# starting with `surveilr-` and sends them to surveilr to execute and store.
$ deno eval "Deno.readTextFile('package.json').then(text => { \
      const data = JSON.parse(text);                          \
      console.log(                                            \
        Object.entries(data.scripts ?? {})                    \
          .filter(([k]) => k.startsWith('surveilr-'))         \
          .map(([k, v]) => ({ [k]: v }))                      \
          .map((line) => JSON.stringify(line)).join('\n'),    \
      )                                                       \
    }).catch((err) => console.error(err));"                   \
  | surveilr ingest tasks

# multiple commands whether each line can be a JSONL formatted object;
# the following runs Deno to grab a remote deno.jsonc file, extract all tasks
# starting with `surveilr-` and sends them to surveilr to execute and store.
$ deno eval "fetch(                                                                  \
      'https://raw.githubusercontent.com/netspective-labs/sql-aide/main/deno.jsonc', \
    ).then((res) => res.json()).then((data) =>                                       \
      console.log(                                                                   \
        Object.entries(data.tasks ?? {})                                             \
          .filter(([k]) => k.startsWith('surveilr-'))                                \
          .map(([k, v]) => ({ [k]: v }))                                             \
          .map((line) => JSON.stringify(line)).join('\n'),                           \
      )                                                                              \
    ).catch((err) => console.error(err));"                                           \
  | surveilr ingest tasks
```

### Testing shell tasks

If you want to test the output of shell tasks without persisting with
`ingest tasks`:

```bash
$ surveilr capturable-exec test task --help
$ surveilr capturable-exec test task -t 'osqueryi "select * from users" --json'
$ surveilr capturable-exec test task -t 'osqueryi "select * from users"'
$ surveilr capturable-exec test task -t '{ "osquery result as plain text": "osqueryi \"SELECT * from users\" --json" }'
$ surveilr capturable-exec test task -t '{ "osquery result as plain text": "osqueryi \"SELECT * from users\"", "nature": "text/plain" }'

$ echo 'osqueryi "select * from users" --json' | surveilr capturable-exec test task --stdin
$ echo '{ "osquery result as plain text": "osqueryi \"SELECT * from users\"", "nature": "text/plain" }' | surveilr capturable-exec test task --stdin
$ cat support/test-fixtures/synthetic-tasks-via-stdin | surveilr capturable-exec test task --stdin
```

See examples
[in this test fixture](support/test-fixtures/synthetic-tasks-via-stdin).

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

When `ingest` command runs, it's main job is to find files and store them in
`uniform_resource` table as records. If the content of a file is already stored
in the file system this works well. However, sometimes we need to generate the
content of a file (or group of files) and store the output of the generated
files. That's where the idea of _Capturable Executables_ (`CEs`) comes in.

CEs allow you to pass in arguments or behaviors to the `ingest` command that
allows certain patterns of files to be executed in a safe shell, and their
STDOUT and STDERR captured and stored in `uniform_resource`. These scripts are
referred to as _capturable executables_ or `CE`s and are influenced through
_Processing Instructions_ (`PI`s) in file names.

For example, if we want `ingest`, as it encounters a `abc.surveilr.sh` file to
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
  "surveilr-ingest": {
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
capturable executables by using `surveilr capturable-exec` subcommands.

```bash
$ surveilr capturable-exec ls --help                    # see all the options (arguments are same as `ingest`)
$ surveilr capturable-exec ls                           # scan for CEs and show a table of what's found
$ surveilr capturable-exec ls --markdown > capturable-exec.md  # find CEs, try to execute them, store their output in a Markdown
```

Running `capturable-exec ls` should show something similar to this:

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

Running `capturable-exec ls --markdown` generates a Markdown document that you
can use to learn more about what `STDIN`, `STDOUT`, and `STDERR` streams will be
created during `ingest`.

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

- `surveilr ingest --captured-fs-exec-sql` Regexp(s) control which files are
  considered capturable executables who output will be captured and executed as
  "SQL batch"
- `surveilr ingest --capture-fs-exec` Regexp(s) control which files are
  considered capturable executables

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

## Email Ingestion

The `surveilr ingest imap` command faclitates the ingestion of emails from a single email address into a queryable SQL format. It enables conversion of emails from specified folders in the mailbox into structured data, enhancing the ability to analyze and query email content directly within the already provided RSSD.

- **Data Transformation**: The command converts the original text of the email stored in the `ur_ingest_session_imap_acct_folder` table. For emails having a  text/html section, it transforms them into a valid, queryable JSON format, making it easier to perform SQL queries on email content.
- **Supported Email Services**: Currently, the command supports Gmail and personal Outlook accounts only.
- **App Passwords**: The password must be an App Password for authentication instead of the account's primary password. App Passwords provide a secure way of accessing your account through third-party applications. For guidance on creating an App Password, please refer [here]() to learn how to create app passwords.

Support for attachements will be in the next release
### Examples
```bash
$ surveilr ingest imap -u user@outlook.com -p 'apppassword' -a "outlook.office365.com" -f INBOX -f SENT
$ surveilr ingest imap -u user@gmail.com -p 'apppassword' -a "imap.gmail.com" -f INBOX --max-no-messages=10000
```

## Microsoft 365
For enterprise Microsoft accounts, app passwords have been disabled and emails can only be accessed through an oauth method. `surveilr` now supports signing in to an enterprise account through two main methods.

**Note**: Before utilizing the new authentication methods, you'll need to register an application in the Azure App Directory. This process involves generating a `client_id` and `client_secret`, which are essential for authenticating with Microsoft 365.
  - Navigate to your Azure console and access the Azure App Directory.
  - Create a new application registration.
  - If you intend to use the Device Code method on headless devices, ensure the application is configured to support them.

### Authentication Methods
Surveilr supports two primary methods for signing into your enterprise account:

1. **Authentication Code**: Ideal for scenarios where `surveilr` is running on a device with browser access. This method involves Surveilr initiating an authentication process in a browser, facilitating the secure login and subsequent email fetching from the Microsoft 365 inbox.

- When setting up the redirect_uri in Azure, it must match the one used in Surveilr and adhere to HTTPS protocols.
- For local or demonstration purposes, tools like ngrok or nginx can be used to proxy requests to the designated port.

```bash
surveilr ingest imap microsoft-365 -i="<client_id>" -s="<client_secret>" -m auth-code -a "https://641c92d93b6f.ngrok.app"
```
In the event that you don't want to pass your credentials in the terminal, `surveilr` provides a utility command to emit the credentials and then you can source them into your current shell.
```bash
$ surveilr admin credentials microsoft-365 -i="<client_id>" -s"<client_secret>" -r="https://641c92d93b6f.ngrok.app" --env ## Prints the credentials to stdout
```

```bash
$ surveilr admin credentials microsoft-365 -i="<client_id>" -s"<client_secret>" -r="https://641c92d93b6f.ngrok.app" --export | source  ## adds the credentials as env variables to the current shell execution
```
then
```bash
surveilr ingest imap microsoft-365 -m auth-code
```

2. **Device Code**: Provides a user-friendly way to authenticate without direct browser access on the host machine. This method displays a code and URL, prompting the user to manually complete the authentication process.

```bash
surveilr ingest imap microsoft-365 -i="<client_id>" -m device-code
```
If you've added the credntials to the current shell:
```bash
surveilr ingest imap microsoft-365 -m device-code
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
`ingest`, etc) and ask it questions to generate SQL for you. The second one is a
good way to help ChatGPT or other LLM to understand the `surveilr` notebooks
schema and ask it questions to generate SQL specifically for the _code
notebooks_ capability.

## SQLPage

[SQLPage](https://github.com/lovasoa/SQLpage) is a unique tool designed for creating SQL-focused web applications with ease. It serves as a straightforward and efficient way to build and deploy web applications that interact directly with your SQL database.

### Getting Started with SQLPage

SQLPage simplifies the process of setting up a web application connected to a SQL database. Surveilr comes with a default configuration table, named `sqlpage_files`, which is automatically included in all RSSD. This table serves as the initial setup for SQLPage.

```bash
surveilr sql-page -p 5556 
```

Navigate to <http://localhost:5556>.

### Updating Content in SQLPage

To update or add new content to SQLPage, you need to modify the SQL statements in the content column of the `sqlpage_files` database table. SQLPage is designed to reflect changes made to these SQL statements in real-time on the web interface.

However, it's important to note that SQLPage relies on timestamp updates to recognize changes. Therefore, every time you modify or add SQL content, you must also update the timestamp associated with these entries. This is a critical step as SQLPage only reloads and displays files that have a registered change in their timestamp.

## UDI-PGP

Check out the documentation [here](./src/udi_pgp/README.md)

## Contributing

Check out [CONTRIBUTING.md](CONTRIBUTING.md).
