import { SQLa, SQLa_tp as tp, whitespace as ws } from "./deps.ts";

// we want to auto-unindent our string literals and remove initial newline
export const markdown = (
  literals: TemplateStringsArray,
  ...expressions: unknown[]
) => {
  const literalSupplier = ws.whitespaceSensitiveTemplateLiteralSupplier(
    literals,
    expressions,
    {
      unindent: true,
      removeInitialNewLine: true,
    },
  );
  let interpolated = "";

  // Loop through each part of the template
  for (let i = 0; i < literals.length; i++) {
    interpolated += literalSupplier(i); // Add the string part
    if (i < expressions.length) {
      interpolated += expressions[i]; // Add the interpolated value
    }
  }
  return interpolated;
};

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
export function codeNotebooksModels<
  EmitContext extends SQLa.SqlEmitContext,
>() {
  const modelsGovn = modelsGovernance<EmitContext>();
  const { keys: gk, domains: gd, model: gm } = modelsGovn;

  const codeNotebookKernel = gm.textPkTable("code_notebook_kernel", {
    code_notebook_kernel_id: gk.textPrimaryKey(),
    kernel_name: gd.text(),
    description: gd.textNullable(),
    mime_type: gd.textNullable(),
    file_extn: gd.textNullable(),
    elaboration: gd.jsonTextNullable(),
    governance: gd.jsonTextNullable(),
    ...gm.housekeeping.columns, // activity_log should store previous versions in JSON format (for history tracking)
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("kernel_name"),
      ];
    },
    populateQS: (t, c, _, tableName) => {
      t.description = markdown`
        A Notebook is a group of Cells. A kernel is a computational engine that executes the code contained in a notebook cell. 
        Each notebook is associated with a kernel of a specific programming language or code transformer which can interpret
        code and produce a result. For example, a SQL notebook might use a SQLite kernel for running SQL code and an AI Prompt
        might prepare AI prompts for LLMs.`;
      c.code_notebook_kernel_id.description =
        `${tableName} primary key and internal label (not a ULID)`;
      c.kernel_name.description = `the kernel name for human/display use cases`;
      c.description.description =
        `any further description of the kernel for human/display use cases`;
      c.mime_type.description =
        `MIME type of this kernel's code in case it will be served`;
      c.file_extn.description =
        `the typical file extension for these kernel's codebases, can be used for syntax highlighting, etc.`;
      c.elaboration.description = `kernel-specific attributes/properties`;
      c.governance.description = `kernel-specific governance data`;
    },

    qualitySystem: {
      description: markdown`
          A Notebook is a group of Cells. A kernel is a computational engine that executes the code contained in a notebook cell. 
          Each notebook is associated with a kernel of a specific programming language or code transformer which can interpret
          code and produce a result. For example, a SQL notebook might use a SQLite kernel for running SQL code and an AI Prompt
          might prepare AI prompts for LLMs.`,
    },
  });

  // Stores all notebook cells in the database so that once the database is
  // created, all SQL is part of the database and may be executed like this
  // from the CLI:
  //    sqlite3 xyz.db "select sql from code_notebook_cell where code_notebook_cell_id = 'infoSchemaMarkdown'" | sqlite3 xyz.db
  // You can pass in arguments using .parameter or `sql_parameters` table, like:
  //    echo ".parameter set X Y; $(sqlite3 xyz.db \"SELECT sql FROM code_notebook_cell where code_notebook_cell_id = 'init'\")" | sqlite3 xyz.db
  const codeNotebookCell = gm.textPkTable("code_notebook_cell", {
    code_notebook_cell_id: gk.textPrimaryKey(),
    notebook_kernel_id: codeNotebookKernel.references.code_notebook_kernel_id(),
    notebook_name: gd.text(),
    cell_name: gd.text(),
    cell_governance: gd.jsonTextNullable(),
    interpretable_code: gd.text(),
    interpretable_code_hash: gd.text(),
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
    populateQS: (t, c, _, tableName) => {
      t.description = markdown`
        Each Notebook is divided into cells, which are individual units of interpretable code.
        The content of Cells depends on the Notebook Kernel and contain the source code to be
        executed by the Notebook's Kernel. The output of the code (text, graphics, etc.) can be
        stateless or may be stateful and store its results and state transitions in code_notebook_state.`;
      c.code_notebook_cell_id.description = `${tableName} primary key`;
      c.cell_governance.description =
        `any idempotency, versioning, hash, branch, tag or other "governance" data (dependent on the cell)`;
    },
  });

  const codeNotebookState = gm.textPkTable("code_notebook_state", {
    code_notebook_state_id: gk.textPrimaryKey(),
    code_notebook_cell_id: codeNotebookCell.references
      .code_notebook_cell_id(),
    from_state: gd.text(),
    to_state: gd.text(),
    transition_result: gd.jsonTextNullable(),
    transition_reason: gd.textNullable(),
    transitioned_at: gd.createdAt(),
    elaboration: gd.jsonTextNullable(),
    ...gm.housekeeping.columns, // activity_log should store previous versions in JSON format (for history tracking)
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("code_notebook_cell_id", "from_state", "to_state"),
      ];
    },
    populateQS: (t, c, _, tableName) => {
      t.description = markdown`
        Records the state of a notebook's cells' executions, computations, and results for Kernels that are stateful. 
        For example, a SQL Notebook Cell that creates tables should only be run once (meaning it's statefule). 
        Other Kernels might store results for functions and output defined in one cell can be used in later cells.`;
      c.code_notebook_state_id.description = `${tableName} primary key`;
      c.code_notebook_cell_id.description =
        `${codeNotebookCell.tableName} row this state describes`;
      c.from_state.description =
        `the previous state (set to "INITIAL" when it's the first transition)`;
      c.to_state.description =
        `the current state; if no rows exist it means no state transition occurred`;
      c.transition_result.description =
        `if the result of state change is necessary for future use`;
      c.transition_reason.description =
        `short text or code explaining why the transition occurred`;
      c.transitioned_at.description = `when the transition occurred`;
      c.elaboration.description =
        `any elaboration needed for the state transition`;
    },
  });

  const informationSchema = {
    tables: [codeNotebookKernel, codeNotebookCell, codeNotebookState],
    tableIndexes: [
      ...codeNotebookKernel.indexes,
      ...codeNotebookCell.indexes,
      ...codeNotebookState.indexes,
    ],
  };

  return {
    modelsGovn,
    codeNotebookKernel,
    codeNotebookCell,
    codeNotebookState,
    informationSchema,
  };
}

export function serviceModels<EmitContext extends SQLa.SqlEmitContext>() {
  const codeNbModels = codeNotebooksModels<EmitContext>();
  const { domains: gd, model: gm } = codeNbModels.modelsGovn;
  const UNIFORM_RESOURCE = "uniform_resource" as const;

  /**
   * Immutable Devices table represents different machines, servers, or workstations.
   * Every device has a unique identifier (ULID) and contains fields for its name,
   * operating system, os_info (possibly Unix name info, like output of uname -a),
   * and a JSON-structured field for additional details about the device.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const device = gm.textPkTable("device", {
    device_id: gm.keys.ulidPrimaryKey(),
    name: gd.text(),
    state: gd.jsonText(),
    boundary: gd.text(),
    segmentation: gd.jsonTextNullable(),
    state_sysinfo: gd.jsonTextNullable(),
    elaboration: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("name", "state", "boundary"),
      ];
    },
    indexes: (props, tableName) => {
      const tif = SQLa.tableIndexesFactory(tableName, props);
      return [tif.index({ isIdempotent: true }, "name", "state")];
    },
    populateQS: (t, c) => {
      t.description =
        `Identity, network segmentation, and sysinfo for devices on which ${UNIFORM_RESOURCE} are found`;
      c.name.description = "unique device identifier (defaults to hostname)";
      c.state.description =
        `should be "SINGLETON" if only one state is allowed, or other tags if multiple states are allowed`;
      c.boundary.description =
        "can be IP address, VLAN, or any other device name differentiator";
      c.segmentation.description = "zero trust or other network segmentation";
      c.state_sysinfo.description =
        "any sysinfo or other state data that is specific to this device (mutable)";
      c.elaboration.description =
        "any elaboration needed for the device (mutable)";
    },
  });

  /**
   * Immutable FileSystem Walk Sessions Represents a single file system scan (or
   * "walk") session. Each time a directory is scanned for files and entries, a
   * record is created here. It includes a reference to the device being scanned
   * and the root path of the scan.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const urWalkSession = gm.textPkTable("ur_walk_session", {
    ur_walk_session_id: gm.keys.ulidPrimaryKey(),
    device_id: device.references.device_id(),
    walk_started_at: gd.dateTime(),
    walk_finished_at: gd.dateTimeNullable(),
    ignore_paths_regex: gd.textNullable(),
    blobs_regex: gd.textNullable(),
    digests_regex: gd.textNullable(),
    elaboration: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("device_id", "created_at"),
      ];
    },
    populateQS: (t, _c, _cols, tableName) => {
      t.description = markdown`
        Immutable FileSystem Walk Sessions Represents a single file system scan (or
        "walk") session. Each time a directory is scanned for files and entries, a
        record is created here. ${tableName} has a foreign key reference to the
        device table so that the same device can be used for multiple walk sessions
        but also the walk sessions can be merged across workstations / servers for easier
        detection of changes and similaries between file systems on different devices.`;
    },
  });

  /**
   * Immutable FileSystem Walk Sessions Represents a single file system scan (or
   * "walk") session. Each time a directory is scanned for files and entries, a
   * record is created here. It includes a reference to the device being scanned
   * and the root path of the scan.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const urWalkSessionPath = gm.textPkTable("ur_walk_session_path", {
    ur_walk_session_path_id: gm.keys.ulidPrimaryKey(),
    walk_session_id: urWalkSession.references
      .ur_walk_session_id(),
    root_path: gd.text(),
    elaboration: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
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
    populateQS: (t, _c, _cols, _tableName) => {
      t.description = markdown`
        Stores file system content walk paths associated with a particular
        session and may have one or more root paths per session.`;
    },
  });

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
  const uniformResource = gm.textPkTable(UNIFORM_RESOURCE, {
    uniform_resource_id: gm.keys.ulidPrimaryKey(),
    device_id: device.references.device_id(), // present in this device
    walk_session_id: urWalkSession.references.ur_walk_session_id(), // introduced in this session
    walk_path_id: urWalkSessionPath.references.ur_walk_session_path_id(), // introduced in this walk path
    uri: gd.text(),
    content_digest: gd.text(), // '-' when no hash was computed (not NULL); content_digest for symlinks will be the same as their target
    content: gd.blobTextNullable(),
    nature: gd.textNullable(), // file extension or MIME
    size_bytes: gd.integerNullable(), // file_bytes for symlinks will be different than their target
    last_modified_at: gd.integerNullable(),
    content_fm_body_attrs: gd.jsonTextNullable(), // each component of frontmatter-based content ({ frontMatter: '', body: '', attrs: {...} })
    frontmatter: gd.jsonTextNullable(), // meta data or other "frontmatter" in JSON format
    elaboration: gd.jsonTextNullable(), // anything that doesn't fit above
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      // TODO: note that content_hash for symlinks will be the same as their target
      //       figure out whether we need anything special in the UNIQUE index
      return [
        c.unique(
          "device_id",
          "content_digest", // use something like `-` when hash is no computed
          "uri",
          "size_bytes",
          "last_modified_at",
        ),
      ];
    },
    indexes: (props, tableName) => {
      const tif = SQLa.tableIndexesFactory(tableName, props);
      return [
        tif.index({ isIdempotent: true }, "device_id", "uri"),
      ];
    },
    populateQS: (t, _c, _cols, tableName) => {
      t.description = markdown`
        Records file system content information. On multiple executions,
        ${tableName} are inserted only if the the file content (see unique 
        index for details). For historical logging, ${tableName} has foreign
        key references to both ${urWalkSession.tableName} and ${urWalkSessionPath.tableName}
        tables to indicate which particular session and walk path the
        resourced was inserted during.`;
    },
  });

  /**
   * Immutable File Content walk path entry table represents an entry that was
   * traversed during path walking of a physical file system (`fs`). This table
   * contains references to the device where the file resides, and references
   * to the file content, digest hash, etc.
   *
   * If you want to see which files did not change between sessions, just "diff"
   * the rows in SQL.
   *
   * Always append new records. NEVER delete or update existing records.
   */
  const urWalkSessionPathFsEntry = gm.textPkTable(
    "ur_walk_session_path_fs_entry",
    {
      ur_walk_session_path_fs_entry_id: gm.keys.ulidPrimaryKey(),
      walk_session_id: urWalkSession.references
        .ur_walk_session_id(),
      walk_path_id: urWalkSessionPath.references.ur_walk_session_path_id(),
      uniform_resource_id: uniformResource.references.uniform_resource_id()
        .optional(),
      file_path_abs: gd.text(),
      file_path_rel_parent: gd.text(),
      file_path_rel: gd.text(),
      file_basename: gd.text(),
      file_extn: gd.textNullable(),
      ur_status: gd.textNullable(), // either NULL if uniform_resource_id was created, or "ERROR"
      ur_status_explanation: gd.jsonTextNullable(),
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
      populateQS: (t, _c, _cols, tableName) => {
        t.description = markdown`
          Contains entries related to file system content walk paths. On multiple executions,
          unlike ${uniformResource.tableName}, ${tableName} rows are always inserted and 
          references the ${uniformResource.tableName} primary key of its related content.
          This method allows for a more efficient query of file version differences across
          sessions. With SQL queries, you can detect which sessions have a file added or modified, 
          which sessions have a file deleted, and what the differences are in file contents
          if they were modified across sessions.`;
      },
    },
  );

  const informationSchema = {
    tables: [
      device,
      urWalkSession,
      urWalkSessionPath,
      uniformResource,
      urWalkSessionPathFsEntry,
    ],
    tableIndexes: [
      ...device.indexes,
      ...urWalkSession.indexes,
      ...urWalkSessionPath.indexes,
      ...uniformResource.indexes,
      ...urWalkSessionPathFsEntry.indexes,
    ],
  };

  return {
    codeNbModels,
    device,
    urWalkSession,
    urWalkSessionPath,
    uniformResource,
    urWalkSessionPathFsEntry,
    informationSchema,
  };
}
