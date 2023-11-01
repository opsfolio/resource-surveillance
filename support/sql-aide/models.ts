import { SQLa, SQLa_tp as tp } from "./deps.ts";

/**
 * Encapsulate the keys, domains, templateState, and other model "governance"
 * needed by the models and notebooks. Instead of saying "types" we use the
 * term "governance".
 * @returns governed keys, domains, template, and context generator for SQLa models
 */
export function modelsGovernance<EmitContext extends SQLa.SqlEmitContext>() {
  type DomainQS = tp.TypicalDomainQS;
  type DomainsQS = tp.TypicalDomainsQS;
  const templateState = tp.governedTemplateState<
    DomainQS,
    DomainsQS,
    EmitContext
  >();
  const sqlEmitContext = <EmitContext extends SQLa.SqlEmitContext>() =>
    SQLa.typicalSqlEmitContext({
      sqlDialect: SQLa.sqliteDialect(),
    }) as EmitContext;
  return {
    keys: tp.governedKeys<DomainQS, DomainsQS, EmitContext>(),
    domains: tp.governedDomains<DomainQS, DomainsQS, EmitContext>(),
    templateState,
    sqlEmitContext,
    model: tp.governedModel<DomainQS, DomainsQS, EmitContext>(
      templateState.ddlOptions,
    ),
  };
}

/**
 * Encapsulate all models that are universally applicable and not specific to
 * this particular service. TODO: consider extracting this into its own pattern.
 * @returns
 */
export function storedNotebookModels<
  EmitContext extends SQLa.SqlEmitContext,
>() {
  const modelsGovn = modelsGovernance<EmitContext>();
  const { keys: gk, domains: gd, model: gm } = modelsGovn;

  // Stores all notebook cells in the database so that once the database is
  // created, all SQL is part of the database and may be executed like this
  // from the CLI:
  //    sqlite3 xyz.db "select sql from sql_notebook_cell where sql_notebook_cell_id = 'infoSchemaMarkdown'" | sqlite3 xyz.db
  // You can pass in arguments using .parameter or `sql_parameters` table, like:
  //    echo ".parameter set X Y; $(sqlite3 xyz.db \"SELECT sql FROM sql_notebook_cell where sql_notebook_cell_id = 'init'\")" | sqlite3 xyz.db
  const storedNotebook = gm.textPkTable("stored_notebook_cell", {
    stored_notebook_cell_id: gk.textPrimaryKey(),
    notebook_name: gd.text(),
    cell_name: gd.text(),
    // TODO: how should we track dependencies so that we only run cells when a condition is met or skip if not?
    // e.g., if a state transition has already occurred, should the script not be run? could we generate `just` or similar task runner?
    //       if_sql: gd.textNullable(), // only run this cell if the given `if_sql` returns a non-zero result (e.g. check state, look at information_schema, etc.)
    //       unless_sql: gd.textNullable(), // only run this cell unless the given `unless_sql` returns a non-zero result
    cell_governance: gd.jsonTextNullable(), // any idempotency, versioning, hash, branch, tag or other "governance" data (dependent on the cell)
    code_interpreter: gd.textNullable(), // SQL by default, shebang-style for others
    interpretable_code: gd.text(),
    interpretable_code_hash: gd.text(),
    interpretable_code_mime_type: gd.textNullable(), // MIME type of the code in case it will be served
    interpretable_code_file_extn: gd.textNullable(), // the typical file extension, can be used for syntax highlighting, etc.
    description: gd.textNullable(),
    arguments: gd.jsonTextNullable(),
    ...gm.housekeeping.columns, // activity_log should store previous versions in JSON format (for history tracking)
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("notebook_name", "cell_name", "interpretable_code_hash"),
      ];
    },
  });

  const storedNotebookState = gm.textPkTable("stored_notebook_state", {
    stored_notebook_state_id: gk.textPrimaryKey(),
    stored_notebook_cell_id: storedNotebook.references
      .stored_notebook_cell_id(),
    from_state: gd.text(), // the previous state (set to "INITIAL" when it's the first transition)
    to_state: gd.text(), // the current state; if no rows exist it means no state transition occurred
    transition_reason: gd.textNullable(), // short text or code explaining why the transition occurred
    transitioned_at: gd.createdAt(), // stores when the transition occurred
    elaboration: gd.jsonTextNullable(), // any elaboration needed for the state transition
    ...gm.housekeeping.columns, // activity_log should store previous versions in JSON format (for history tracking)
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("stored_notebook_cell_id", "from_state", "to_state"),
      ];
    },
  });

  const informationSchema = {
    tables: [storedNotebook, storedNotebookState],
    tableIndexes: [
      ...storedNotebook.indexes,
      ...storedNotebookState.indexes,
    ],
  };

  return {
    modelsGovn,
    storedNotebook,
    storedNotebookState,
    informationSchema,
  };
}

export function serviceModels<EmitContext extends SQLa.SqlEmitContext>() {
  const storedNbModels = storedNotebookModels<EmitContext>();
  const { keys: gk, domains: gd, model: gm } = storedNbModels.modelsGovn;

  /**
   * Immutable Devices table represents different machines, servers, or workstations.
   * Every device has a unique identifier (ULID) and contains fields for its name,
   * operating system, os_info (possibly Unix name info, like output of uname -a),
   * and a JSON-structured field for additional details about the device.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const mimeType = gm.textPkTable(
    "mime_type",
    {
      mime_type_id: gk.ulidPrimaryKey(), // TODO: allow setting default to `ulid()` type like autoIncPK execpt autoUlidPK or something
      name: gd.text(),
      description: gd.text(),
      file_extn: gd.text(),
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      constraints: (props, tableName) => {
        const c = SQLa.tableConstraints(tableName, props);
        return [
          c.unique("name", "file_extn"),
        ];
      },
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [tif.index({ isIdempotent: true }, "file_extn")];
      },
    },
  );

  /**
   * Immutable Devices table represents different machines, servers, or workstations.
   * Every device has a unique identifier (ULID) and contains fields for its name,
   * operating system, os_info (possibly Unix name info, like output of uname -a),
   * and a JSON-structured field for additional details about the device.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const device = gm.textPkTable(
    "device",
    {
      device_id: gm.keys.ulidPrimaryKey(), // TODO: allow setting default to `ulid()` type like autoIncPK execpt autoUlidPK or something
      name: gd.text(),
      boundary: gd.text(), // can be IP address, VLAN, or any other device name differentiator
      device_elaboration: gd.jsonTextNullable(),
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      constraints: (props, tableName) => {
        const c = SQLa.tableConstraints(tableName, props);
        return [
          c.unique("name", "boundary"),
        ];
      },
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [tif.index({ isIdempotent: true }, "name")];
      },
    },
  );

  /**
   * Immutable FileSystem Walk Sessions Represents a single file system scan (or
   * "walk") session. Each time a directory is scanned for files and entries, a
   * record is created here. It includes a reference to the device being scanned
   * and the root path of the scan.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const fsContentWalkSession = gm.textPkTable(
    "fs_content_walk_session",
    {
      fs_content_walk_session_id: gm.keys.ulidPrimaryKey(),
      device_id: device.references.device_id(),
      walk_started_at: gd.dateTime(),
      walk_finished_at: gd.dateTimeNullable(),
      max_fileio_read_bytes: gd.integer(),
      ignore_paths_regex: gd.textNullable(),
      blobs_regex: gd.textNullable(),
      digests_regex: gd.textNullable(),
      elaboration: gd.jsonTextNullable(),
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      constraints: (props, tableName) => {
        const c = SQLa.tableConstraints(tableName, props);
        return [
          c.unique("device_id", "created_at"),
        ];
      },
    },
  );

  /**
   * Immutable FileSystem Walk Sessions Represents a single file system scan (or
   * "walk") session. Each time a directory is scanned for files and entries, a
   * record is created here. It includes a reference to the device being scanned
   * and the root path of the scan.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const fsContentWalkPath = gm.textPkTable(
    "fs_content_walk_path",
    {
      fs_content_walk_path_id: gm.keys.ulidPrimaryKey(),
      walk_session_id: fsContentWalkSession.references
        .fs_content_walk_session_id(),
      root_path: gd.text(),
      elaboration: gd.jsonTextNullable(),
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      constraints: (props, tableName) => {
        const c = SQLa.tableConstraints(tableName, props);
        return [
          c.unique("walk_session_id", "root_path", "created_at"),
        ];
      },
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [
          tif.index({ isIdempotent: true }, "walk_session_id", "root_path"),
        ];
      },
    },
  );

  /**
   * Immutable File Content table represents the content and metadata of a file at
   * a particular point in time. This table contains references to the device where
   * the file resides, file content (optional), digest hash of the content (to
   * detect changes), and modification time.
   *
   * The file content is "versioned" using mtime which are then related to the walk
   * session to see which version.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const fsContent = gm.textPkTable(
    "fs_content",
    {
      fs_content_id: gm.keys.ulidPrimaryKey(),
      walk_session_id: fsContentWalkSession.references
        .fs_content_walk_session_id(),
      walk_path_id: fsContentWalkPath.references.fs_content_walk_path_id(),
      file_path: gd.text(),
      content_digest: gd.text(), // '-' when no hash was computed (not NULL); content_digest for symlinks will be the same as their target
      content: gd.blobTextNullable(),
      file_bytes: gd.integerNullable(), // file_bytes for symlinks will be different than their target
      file_extn: gd.textNullable(),
      file_mode: gd.integerNullable(),
      file_mode_human: gd.textNullable(),
      file_mtime: gd.integerNullable(),
      content_fm_body_attrs: gd.jsonTextNullable(), // each component of frontmatter-based content ({ frontMatter: '', body: '', attrs: {...} })
      frontmatter: gd.jsonTextNullable(), // meta data or other "frontmatter" in JSON format
      elaboration: gd.jsonTextNullable(), // anything that doesn't fit above
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      constraints: (props, tableName) => {
        const c = SQLa.tableConstraints(tableName, props);
        // TODO: note that content_hash for symlinks will be the same as their target
        //       figure out whether we need anything special in the UNIQUE index
        return [
          c.unique(
            "content_digest", // use something like `-` when hash is no computed
            "file_path",
            "file_bytes",
            "file_mtime",
          ),
        ];
      },
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [
          tif.index({ isIdempotent: true }, "walk_session_id", "file_path"),
        ];
      },
    },
  );

  /**
   * Immutable File Content walk path entry table represents an entry that was
   * traversed during path walking. This table contains references to the device
   * where the file resides, and references to the file content, digest hash, etc.
   *
   * If you want to see which files did not change between sessions, just "diff"
   * the rows in SQL.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const fsContentWalkPathEntry = gm.textPkTable(
    "fs_content_walk_path_entry",
    {
      fs_content_walk_path_entry_id: gm.keys.ulidPrimaryKey(),
      walk_session_id: fsContentWalkSession.references
        .fs_content_walk_session_id(),
      walk_path_id: fsContentWalkPath.references.fs_content_walk_path_id(),
      fs_content_id: fsContent.references.fs_content_id().optional(),
      file_path_abs: gd.text(),
      file_path_rel_parent: gd.text(),
      file_path_rel: gd.text(),
      file_basename: gd.text(),
      file_extn: gd.textNullable(),
      elaboration: gd.jsonTextNullable(), // anything that doesn't fit above
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [
          tif.index({ isIdempotent: true }, "walk_session_id", "file_path_abs"),
        ];
      },
    },
  );

  const informationSchema = {
    tables: [
      mimeType,
      device,
      fsContentWalkSession,
      fsContentWalkPath,
      fsContent,
      fsContentWalkPathEntry,
    ],
    tableIndexes: [
      ...mimeType.indexes,
      ...device.indexes,
      ...fsContentWalkSession.indexes,
      ...fsContentWalkPath.indexes,
      ...fsContent.indexes,
      ...fsContentWalkPathEntry.indexes,
    ],
  };

  return {
    storedNbModels,
    mimeType,
    device,
    fsContentWalkSession,
    fsContentWalkPath,
    fsContent,
    fsContentWalkPathEntry,
    informationSchema,
  };
}
