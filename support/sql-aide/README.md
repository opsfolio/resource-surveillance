# SQL Aide (`SQLa`) Notebooks

This directory is a SQL generator which produces all the SQL used by our utility
in consistent, high-quality manner.

## Notebook Cells for Script Storage & Execution Tracking

The `code_notebook_cell` table serves as a comprehensive repository to store
and manage SQL scripts of all kinds, including but not limited to:

- `SQL DDL`: For database structure modifications such as `CREATE`, `ALTER`, and
  `DROP`.
- `SQL DQL`: Scripts related to data querying like `SELECT` operations.
- `SQL DML`: Scripts for data manipulation including `INSERT`, `UPDATE`, and
  `DELETE`.
- ... and other SQL operations.

While it's versatile enough to manage various SQL tasks, `code_notebook_cell`
table's primary advantage lies in storing SQL DDL scripts needed for migrations

**Columns**:

- `code_notebook_cell_id`: A unique identifier for each SQL script.
- `notebook_name`: Broad categorization or project name. Especially useful for
  grouping related migration scripts.
- `cell_name`: A descriptive name for the SQL operation or step.
- `cell_governance`: Optional JSON field for any governance or
  compliance-related data.
- `kernel`: The SQL dialect or interpreter the script targets, such as
  PostgreSQL, MySQL, etc. (might also support shebang-style for non SQL)
- `interpretable_code`: The SQL script itself (or any other runtime).
- `description`: A brief description or context regarding the purpose of the
  script.
- `activity_log`: optional JSON which stores the history of the changes to thie
  notebook cell

### Migration Scripts & Database Evolution

One of the best uses for `code_notebook_cell` is to manage SQL DDL scripts
crucial for database migrations. As databases evolve, tracking structural
changes becomes vital. By cataloging these DDL scripts, one can maintain a clear
version history, ensuring that database evolution is orderly, reversible, and
auditable.

To maintain a clear audit trail of script execution, the `code_notebook_state`
table logs each execution's status. And, migration scripts can use the state to
know whether something has already been executed.

**Inserting a New Execution State**:

To log the execution of a script, you can add an entry like so:

```sql
INSERT INTO code_notebook_state 
(code_notebook_state_id, code_notebook_cell_id, from_state, to_state, created_at, created_by)
VALUES
(
    'generated_or_provided_state_id',
    (SELECT code_notebook_cell_id 
     FROM code_notebook_cell 
     WHERE notebook_name = 'specific_project_name' 
     AND cell_name = 'specific_script_name'),
    'INITIAL',
    'EXECUTED',
    CURRENT_TIMESTAMP,
    'executor_name_or_system'
);
```

**State Transitions**:

With `from_state` and `to_state`, you can track a script's lifecycle, from its
`INITIAL` state to any subsequent states like 'EXECUTED', 'ROLLED_BACK', etc.
This provides a traceable path of script interactions.

### CLI Use

One way to keep your code and the database in sync is to just use the database
to get its SQL (instead of putting it into an app) and execute the SQL directly
in the database.

```bash
$ sqlite3 xyz.db "select sql from code_notebook_cell where code_notebook_cell_id = 'infoSchemaMarkdown'" | sqlite3 xyz.db
```

You can pass in arguments using `.parameter` or `sql_parameters` table, like:

```bash
$ echo ".parameter set X Y; $(sqlite3 xyz.db \"SELECT sql FROM code_notebook_cell where code_notebook_cell_id = 'init'\")" | sqlite3 xyz.db
```
