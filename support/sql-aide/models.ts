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

  const assuranceSchema = gm.textPkTable("assurance_schema", {
    assurance_schema_id: gk.varCharPrimaryKey(),
    assurance_type: gd.text(),
    code: gd.text(),
    code_json: gd.jsonTextNullable(),
    governance: gd.jsonTextNullable(),
    ...gm.housekeeping.columns, // activity_log should store previous versions in JSON format (for history tracking)
  }, {
    isIdempotent: true,
    populateQS: (t, c, _, tableName) => {
      t.description = markdown`
        A Notebook is a group of Cells. A kernel is a computational engine that executes the code contained in a notebook cell.
        Each notebook is associated with a kernel of a specific programming language or code transformer which can interpret
        code and produce a result. For example, a SQL notebook might use a SQLite kernel for running SQL code and an AI Prompt
        might prepare AI prompts for LLMs.`;
      c.assurance_schema_id.description =
        `${tableName} primary key and internal label (not a ULID)`;
      c.assurance_type.description = `'JSON Schema', 'XML Schema', etc.`;
      c.code.description =
        `If the schema is other than JSON Schema, use this for the validation code`;
      c.code_json.description =
        `If the schema is a JSON Schema or the assurance code has a JSON representation`;
      c.governance.description =
        `JSON schema-specific governance data (description, documentation, usage, etc. in JSON)`;
    },

    qualitySystem: {
      description: markdown`
          A Notebook is a group of Cells. A kernel is a computational engine that executes the code contained in a notebook cell.
          Each notebook is associated with a kernel of a specific programming language or code transformer which can interpret
          code and produce a result. For example, a SQL notebook might use a SQLite kernel for running SQL code and an AI Prompt
          might prepare AI prompts for LLMs.`,
    },
  });

  const codeNotebookKernel = gm.textPkTable("code_notebook_kernel", {
    code_notebook_kernel_id: gk.varCharPrimaryKey(),
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
    code_notebook_cell_id: gk.varCharPrimaryKey(),
    notebook_kernel_id: codeNotebookKernel.belongsTo.code_notebook_kernel_id(),
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
    code_notebook_state_id: gk.varCharPrimaryKey(),
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
    tables: [
      assuranceSchema,
      codeNotebookKernel,
      codeNotebookCell,
      codeNotebookState,
    ],
    tableIndexes: [
      ...assuranceSchema.indexes,
      ...codeNotebookKernel.indexes,
      ...codeNotebookCell.indexes,
      ...codeNotebookState.indexes,
    ],
  };

  return {
    modelsGovn,
    assuranceSchema,
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

  const device = gm.textPkTable("device", {
    device_id: gm.keys.varCharPrimaryKey(),
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

  const behavior = gm.textPkTable("behavior", {
    behavior_id: gm.keys.varCharPrimaryKey(),
    device_id: device.belongsTo.device_id(),
    behavior_name: gd.text(),
    behavior_conf_json: gd.jsonText(),
    assurance_schema_id: codeNbModels.assuranceSchema.references
      .assurance_schema_id().optional(),
    governance: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("device_id", "behavior_name"),
      ];
    },
    populateQS: (t, c, _cols, tableName) => {
      t.description = markdown`
          Behaviors are configuration "presets" that can be used to drive
          application operations at runtime. For example, ingest behaviors
          include configs that indicate which files to ignore, which to
          scan, when to load content, etc. This is more convenient than
          creating

          ${tableName} has a foreign key reference to the device table since
          behaviors might be device-specific.`;
      c.behavior_name.description =
        `Arbitrary but unique per-device behavior name (e.g. ingest::xyz)`;
      c.behavior_conf_json.description =
        `Configuration, settings, parameters, etc. describing the behavior (JSON, behavior-dependent)`;
      c.governance.description =
        `Descriptions or other "governance" details (JSON, behavior-dependent)`;
    },
  });

  const urIngestPathMatchRule = gm.textPkTable(
    "ur_ingest_resource_path_match_rule",
    {
      ur_ingest_resource_path_match_rule_id: gm.keys.varCharPrimaryKey(),
      namespace: gd.text(),
      regex: gd.text(),
      flags: gd.text(),
      nature: gd.textNullable(),
      priority: gd.textNullable(),
      description: gd.text().optional(),
      elaboration: gd.jsonTextNullable(),
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      constraints: (props, tableName) => {
        const c = SQLa.tableConstraints(tableName, props);
        return [
          c.unique("namespace", "regex"),
        ];
      },
      populateQS: (t, _c, _cols, _tableName) => {
        t.description = markdown`
        A regular expression can determine the flags to apply to an ingestion path
        and if the regular expr contains a nature capture group that pattern match
        will assign the nature too.`;
      },
    },
  );

  const urIngestPathRewriteRule = gm.textPkTable(
    "ur_ingest_resource_path_rewrite_rule",
    {
      ur_ingest_resource_path_rewrite_rule_id: gm.keys.varCharPrimaryKey(),
      namespace: gd.text(),
      regex: gd.text(),
      replace: gd.text(),
      priority: gd.textNullable(),
      description: gd.text().optional(),
      elaboration: gd.jsonTextNullable(),
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      constraints: (props, tableName) => {
        const c = SQLa.tableConstraints(tableName, props);
        return [
          c.unique("namespace", "regex", "replace"),
        ];
      },
      populateQS: (t, _c, _cols, _tableName) => {
        t.description = markdown`
        A regular expression can determine whether certain paths should be
        rewritten before ${urIngestPathMatchRule.tableName} matches occur.`;
      },
    },
  );

  const urIngestSession = gm.textPkTable("ur_ingest_session", {
    ur_ingest_session_id: gm.keys.varCharPrimaryKey(),
    device_id: device.belongsTo.device_id(),
    behavior_id: behavior.belongsTo.behavior_id().optional(),
    behavior_json: gd.jsonTextNullable(),
    ingest_started_at: gd.dateTime(),
    ingest_finished_at: gd.dateTimeNullable(),
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
        Immutable ingestion sessions represents any "discovery" or "walk" operation.
        This could be a device file system scan or any other resource discovery
        session. Each time a discovery operation starts, a record is created.
        ${tableName} has a foreign key reference to the device table so that the
        same device can be used for multiple ingest sessions but also the ingest
        sessions can be merged across workstations / servers for easier detection
        of changes and similaries between file systems on different devices.`;
    },
  });

  const urIngestSessionFsPath = gm.textPkTable("ur_ingest_session_fs_path", {
    ur_ingest_session_fs_path_id: gm.keys.varCharPrimaryKey(),
    ingest_session_id: urIngestSession.belongsTo
      .ur_ingest_session_id(),
    root_path: gd.text(),
    elaboration: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("ingest_session_id", "root_path", "created_at"),
      ];
    },
    indexes: (props, tableName) => {
      const tif = SQLa.tableIndexesFactory(tableName, props);
      return [
        tif.index({ isIdempotent: true }, "ingest_session_id", "root_path"),
      ];
    },
    populateQS: (t, _c, cols, _tableName) => {
      t.description = markdown`
        Immutable ingest session file system path represents a discovery or "walk" path. If
        the session included a file system scan, then ${cols.root_path.identity} is the
        root file system path that was scanned. If the session was discovering
        resources in another target then ${cols.root_path.identity} would be
        representative of the target path (could be a URI).`;
    },
  });

  const uniformResource = gm.textPkTable(UNIFORM_RESOURCE, {
    uniform_resource_id: gm.keys.varCharPrimaryKey(),
    device_id: device.belongsTo.device_id(),
    ingest_session_id: urIngestSession.belongsTo.ur_ingest_session_id(),
    ingest_fs_path_id: urIngestSessionFsPath.references
      .ur_ingest_session_fs_path_id().optional(),
    uri: gd.text(),
    content_digest: gd.text(),
    content: gd.blobTextNullable(),
    nature: gd.textNullable(),
    size_bytes: gd.integerNullable(),
    last_modified_at: gd.dateTimeNullable(),
    content_fm_body_attrs: gd.jsonTextNullable(),
    frontmatter: gd.jsonTextNullable(),
    elaboration: gd.jsonTextNullable(),
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
    populateQS: (t, c, _cols, tableName) => {
      t.description = markdown`
        Immutable resource and content information. On multiple executions,
        ${tableName} are inserted only if the the content (see unique
        index for details). For historical logging, ${tableName} has foreign
        key references to both ${urIngestSession.tableName} and ${urIngestSessionFsPath.tableName}
        tables to indicate which particular session and ingestion path the
        resourced was inserted during.`;
      c.uniform_resource_id.description = `${tableName} ULID primary key`;
      c.device_id.description =
        `which ${device.tableName} row introduced this resource`;
      c.ingest_session_id.description =
        `which ${urIngestSession.tableName} row introduced this resource`;
      c.ingest_fs_path_id.description =
        `which ${urIngestSessionFsPath.tableName} row introduced this resource`;
      c.uri.description =
        `the resource's URI (dependent on how it was acquired and on which device)`;
      c.content_digest.description =
        `'-' when no hash was computed (not NULL); content_digest for symlinks will be the same as their target`;
      c.content.description =
        `either NULL if no content was acquired or the actual blob/text of the content`;
      c.nature.description = `file extension or MIME`;
      c.content_fm_body_attrs.description =
        `each component of frontmatter-based content ({ frontMatter: '', body: '', attrs: {...} })`;
      c.frontmatter.description =
        `meta data or other "frontmatter" in JSON format`;
      c.elaboration.description =
        `anything that doesn't fit in other columns (JSON)`;
    },
  });

  const uniformResourceTransform = gm.textPkTable(
    `uniform_resource_transform`,
    {
      uniform_resource_transform_id: gm.keys.varCharPrimaryKey(),
      uniform_resource_id: uniformResource.belongsTo.uniform_resource_id(),
      uri: gd.text(),
      content_digest: gd.text(),
      content: gd.blobTextNullable(),
      nature: gd.textNullable(),
      size_bytes: gd.integerNullable(),
      elaboration: gd.jsonTextNullable(),
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
            "uniform_resource_id",
            "content_digest", // use something like `-` when hash is no computed
            "nature",
            "size_bytes",
          ),
        ];
      },
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [
          tif.index(
            { isIdempotent: true },
            "uniform_resource_id",
            "content_digest",
          ),
        ];
      },
      populateQS: (t, c, _cols, tableName) => {
        t.description = markdown`
          ${uniformResource.tableName} transformed content`;
        c.uniform_resource_transform_id.description =
          `${tableName} ULID primary key`;
        c.uniform_resource_id.description =
          `${uniformResource.tableName} row ID of original content`;
        c.content_digest.description = `transformed content hash`;
        c.content.description = `transformed content`;
        c.nature.description = `file extension or MIME`;
        c.elaboration.description =
          `anything that doesn't fit in other columns (JSON)`;
      },
    },
  );

  const urIngestSessionFsPathEntry = gm.textPkTable(
    "ur_ingest_session_fs_path_entry",
    {
      ur_ingest_session_fs_path_entry_id: gm.keys.varCharPrimaryKey(),
      ingest_session_id: urIngestSession.belongsTo
        .ur_ingest_session_id(),
      ingest_fs_path_id: urIngestSessionFsPath.belongsTo
        .ur_ingest_session_fs_path_id(),
      uniform_resource_id: uniformResource.references.uniform_resource_id()
        .optional(), // if a uniform_resource was prepared for this or already existed
      file_path_abs: gd.text(),
      file_path_rel_parent: gd.text(),
      file_path_rel: gd.text(),
      file_basename: gd.text(),
      file_extn: gd.textNullable(),
      captured_executable: gd.jsonTextNullable(), // JSON-based details to know what executable was captured, if any
      ur_status: gd.textNullable(), // "CREATED", "EXISTING", "ERROR" / "WARNING" / etc.
      ur_diagnostics: gd.jsonTextNullable(), // JSON diagnostics for ur_status column
      ur_transformations: gd.jsonTextNullable(), // JSON-based details to know what transformations occurred, if any
      elaboration: gd.jsonTextNullable(), // anything that doesn't fit above
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [
          tif.index(
            { isIdempotent: true },
            "ingest_session_id",
            "file_path_abs",
          ),
        ];
      },
      populateQS: (t, _c, _cols, tableName) => {
        t.description = markdown`
          Contains entries related to file system content ingestion paths. On multiple executions,
          unlike ${uniformResource.tableName}, ${tableName} rows are always inserted and
          references the ${uniformResource.tableName} primary key of its related content.
          This method allows for a more efficient query of file version differences across
          sessions. With SQL queries, you can detect which sessions have a file added or modified,
          which sessions have a file deleted, and what the differences are in file contents
          if they were modified across sessions.`;
      },
    },
  );

  const urIngestSessionTaskEntry = gm.textPkTable(
    "ur_ingest_session_task",
    {
      ur_ingest_session_task_id: gm.keys.varCharPrimaryKey(),
      ingest_session_id: urIngestSession.references
        .ur_ingest_session_id(),
      uniform_resource_id: uniformResource.references.uniform_resource_id()
        .optional(), // if a uniform_resource was prepared for this or already existed
      captured_executable: gd.jsonText(),
      ur_status: gd.textNullable(), // "CREATED", "EXISTING", "ERROR" / "WARNING" / etc.
      ur_diagnostics: gd.jsonTextNullable(), // JSON diagnostics for ur_status column
      ur_transformations: gd.jsonTextNullable(), // JSON-based details to know what transformations occurred, if any
      elaboration: gd.jsonTextNullable(), // anything that doesn't fit above
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [
          tif.index(
            { isIdempotent: true },
            "ingest_session_id",
          ),
        ];
      },
      populateQS: (t, _c, _cols, tableName) => {
        t.description = markdown`
          Contains entries related to task content ingestion paths. On multiple executions,
          unlike ${uniformResource.tableName}, ${tableName} rows are always inserted and
          references the ${uniformResource.tableName} primary key of its related content.
          This method allows for a more efficient query of file version differences across
          sessions. With SQL queries, you can detect which sessions have a file added or modified,
          which sessions have a file deleted, and what the differences are in file contents
          if they were modified across sessions.`;
      },
    },
  );

  const urIngestSessionImapAccount = gm.textPkTable("ur_ingest_session_imap_account", {
    ur_ingest_session_imap_account_id: gm.keys.varCharPrimaryKey(),
    ingest_session_id: urIngestSession.belongsTo
      .ur_ingest_session_id(),
    email: gd.text(),
    password: gd.text(),
    host: gd.text(),
    elaboration: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("ingest_session_id", "email"),
      ];
    },
    indexes: (props, tableName) => {
      const tif = SQLa.tableIndexesFactory(tableName, props);
      return [
        tif.index({ isIdempotent: true }, "ingest_session_id", "email"),
      ];
    },
    populateQS: (t, _c, cols, _tableName) => {
      t.description = markdown`
        Immutable ingest session folder system represents an email address to be ingested. Each
        session includes an email, then ${cols.email.identity} is the
        folder that was scanned.`;
    },
  });


  const urIngestSessionImapAcctFolder = gm.textPkTable("ur_ingest_session_imap_acct_folder", {
    ur_ingest_session_imap_acct_folder_id: gm.keys.varCharPrimaryKey(),
    ingest_session_id: urIngestSession.belongsTo
      .ur_ingest_session_id(),
        ingest_account_id: urIngestSessionImapAccount.belongsTo
      .ur_ingest_session_imap_account_id(),
    folder_name: gd.text(),
    elaboration: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    constraints: (props, tableName) => {
      const c = SQLa.tableConstraints(tableName, props);
      return [
        c.unique("ingest_account_id", "folder_name"),
      ];
    },
    indexes: (props, tableName) => {
      const tif = SQLa.tableIndexesFactory(tableName, props);
      return [
        tif.index({ isIdempotent: true }, "ingest_session_id", "folder_name"),
      ];
    },
    populateQS: (t, _c, cols, _tableName) => {
      t.description = markdown`
        Immutable ingest session folder system represents a folder or mailbox in an email account, e.g. "INBOX" or "SENT". Each
        session includes a folder scan, then ${cols.folder_name.identity} is the
        folder that was scanned.`;
    },
  });

  const urIngestSessionImapAcctFolderMessage = gm.textPkTable(
    "ur_ingest_session_imap_acct_folder_message",
    {
      ur_ingest_session_imap_acct_folder_message_id: gm.keys.varCharPrimaryKey(),
      ingest_session_id: urIngestSession.belongsTo
        .ur_ingest_session_id(),
      ingest_imap_acct_folder_id: urIngestSessionImapAcctFolder.belongsTo
        .ur_ingest_session_imap_acct_folder_id(),
      uniform_resource_id: uniformResource.references.uniform_resource_id()
        .optional(), // if a uniform_resource was prepared for this or already existed
      message: gd.text(),
      ...gm.housekeeping.columns,
    },
    {
      isIdempotent: true,
      indexes: (props, tableName) => {
        const tif = SQLa.tableIndexesFactory(tableName, props);
        return [
          tif.index(
            { isIdempotent: true },
            "ingest_session_id",
          ),
        ];
      },
      populateQS: (t, _c, _cols, tableName) => {
        t.description = markdown`
          Contains messages related in a folder that was ingested. On multiple executions,
          unlike ${uniformResource.tableName}, ${tableName} rows are always inserted and
          references the ${uniformResource.tableName} primary key of its related content.
          This method allows for a more efficient query of message version differences across
          sessions. With SQL queries, you can detect which sessions have a messaged added or modified,
          which sessions have a message deleted, and what the differences are in message contents
          if they were modified across sessions.`;
      },
    },
  );

    const urIngestSessionImapAcctFolderMessageAttachment = gm.textPkTable(
    "ur_ingest_session_imap_acct_folder_message_attachment",
    {
      ur_ingest_session_imap_acct_folder_message_attachment_id: gm.keys.varCharPrimaryKey(),
      ingest_session_id: urIngestSession.belongsTo
        .ur_ingest_session_id(),
      ur_ingest_session_imap_acct_folder_message_id: urIngestSessionImapAcctFolderMessage.belongsTo
        .ur_ingest_session_imap_acct_folder_message_id(),
      uniform_resource_id: uniformResource.references.uniform_resource_id()
        .optional(),
      nature: gd.text(),
      message: gd.text(),
      ...gm.housekeeping.columns,
    },
    {
      populateQS: (t, _c, _cols, tableName) => {
        t.description = markdown`
          Contains messages related in a folder that was ingested. On multiple executions,
          unlike ${uniformResource.tableName}, ${tableName} rows are always inserted and
          references the ${uniformResource.tableName} primary key of its related content.
          This method allows for a more efficient query of message version differences across
          sessions. With SQL queries, you can detect which sessions have a messaged added or modified,
          which sessions have a message deleted, and what the differences are in message contents
          if they were modified across sessions.`;
      },
    },
  );


  const informationSchema = {
    tables: [
      device,
      behavior,
      urIngestPathMatchRule,
      urIngestPathRewriteRule,
      urIngestSession,
      urIngestSessionFsPath,
      uniformResource,
      uniformResourceTransform,
      urIngestSessionFsPathEntry,
      urIngestSessionTaskEntry,
      urIngestSessionImapAccount,
      urIngestSessionImapAcctFolder,
      urIngestSessionImapAcctFolderMessage,
      urIngestSessionImapAcctFolderMessageAttachment
    ],
    tableIndexes: [
      ...device.indexes,
      ...behavior.indexes,
      ...urIngestPathMatchRule.indexes,
      ...urIngestPathRewriteRule.indexes,
      ...urIngestSession.indexes,
      ...urIngestSessionFsPath.indexes,
      ...uniformResource.indexes,
      ...uniformResourceTransform.indexes,
      ...urIngestSessionFsPathEntry.indexes,
      ...urIngestSessionTaskEntry.indexes,
      ...urIngestSessionImapAcctFolder.indexes,
      ...urIngestSessionImapAcctFolderMessage.indexes,
      ...urIngestSessionImapAcctFolderMessageAttachment.indexes,
      ...urIngestSessionImapAccount.indexes,
    ],
  };

  return {
    codeNbModels,
    device,
    behavior,
    urIngestPathMatchRule,
    urIngestPathRewriteRule,
    urIngestSession,
    urIngestSessionFsPath,
    uniformResource,
    uniformResourceTransform,
    urIngestSessionFsPathEntry,
    urIngestSessionTaskEntry,
    informationSchema,
    urIngestSessionImapAccount,
    urIngestSessionImapAcctFolder,
    urIngestSessionImapAcctFolderMessage,
    urIngestSessionImapAcctFolderMessageAttachment
  };
}

export function adminModels<
  EmitContext extends SQLa.SqlEmitContext,
>() {
  const modelsGovn = modelsGovernance<EmitContext>();
  const { keys: gk, domains: gd, model: gm } = modelsGovn;

  const udiPgpSet = gm.textPkTable("udi_pgp_set", {
    udi_pgp_set_id: gk.varCharPrimaryKey(),
    query_text: gd.text(),
    generated_ncl: gd.text(),
    status: gd.integer(), //zero or
    status_text: gd.textNullable(),
    elaboration: gd.jsonTextNullable(),
    diagnostics_file: gd.textNullable(),
    diagnostics_file_content: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    populateQS: (t, c, _, tableName) => {
      t.description = markdown`
      Specific table for all SET queries. That is, all configuration queries.
        `;
      c.udi_pgp_set_id.description =
        `${tableName} primary key and it corresponds to the "udi_pgp_observe_query_exec_id" in "udi_pgp_observe_query_exec".`;
      c.query_text.description =
        `The original query passed to UDI-PGP through the SET command`;
      c.generated_ncl.description = `The NCL file generated from the schema`;
      c.status.description =
        `The status of the query. Corresponds to "exec_status" on the "udi_pgp_observe_query_exec" table.`;
      c.status_text.description = `The reason the query failed`;
      c.elaboration.description =
        `A general object containing details like all the events that transpired during the execution of a query`;
      c.diagnostics_file.description = `Location the config query was written to.`;
      c.diagnostics_file_content.description = `Content of the diagnostics file.`;
    },

    qualitySystem: {
      description: markdown`
        A Supplier is any source that can provide data to be retrieved, such as osquery, git.
        Each supplier is stored in this table along with its metadata`,
    },
  });

  const udiPgpSupplier = gm.textPkTable("udi_pgp_supplier", {
    udi_pgp_supplier_id: gk.varCharPrimaryKey(),
    type: gd.text(),
    mode: gd.text(),
    ssh_targets: gd.jsonTextNullable(),
    auth: gd.jsonTextNullable(),
    atc_file_path: gd.textNullable(),
    governance: gd.jsonTextNullable(),
    ...gm.housekeeping.columns,
  }, {
    isIdempotent: true,
    populateQS: (t, c, _, tableName) => {
      t.description = markdown`
        A Supplier is any source that can provide data to be retrieved, such as osquery, git.
        Each supplier is stored in this table along with its metadata.`;
      c.udi_pgp_supplier_id.description =
        `${tableName} primary key and it corresponds to the name of the supplier passed in during creation of the supplier`;
      c.mode.description =
        `Specifies the mode of operation or interaction with the supplier. The modes define whether the supplier is accessed remotely or locally.`;
      c.type.description =
        `Identifies the type of supplier. UDI-PGP currently supports three types of suppliers: osquery, git`;
      c.ssh_targets.description =
        ` Lists the SSH targets for the supplier if the query is to be executed remotely. This field is optional and is relevant only for suppliers that have type of remote.`;
      c.atc_file_path.description =
        `Specifies the file path to an ATC (Auto Table Construction) file associated with a supplier of type "osquery".`;
      c.auth.description =
        `Defines authentication mechanisms or credentials required to interact with the supplier. This field is a vector, allowing for multiple authentication methods`;
      c.governance.description =
        `JSON schema-specific governance data (description, documentation, usage, etc. in JSON)`;
    },

    qualitySystem: {
      description:
        markdown`Get detailed information about the lifecycle of a configuration query.`,
    },
  });

  const udiPgpConfig = gm.textPkTable("udi_pgp_config", {
    udi_pgp_config_id: gk.varCharPrimaryKey(),
    addr: gd.text(),
    health: gd.textNullable(),
    metrics: gd.textNullable(),
    config_ncl: gd.textNullable(),
    admin_db_path: gd.textNullable(),
    surveilr_version: gd.textNullable(),
    governance: gd.jsonTextNullable(),
    ...gm.housekeeping.columns, // activity_log should store previous versions in JSON format (for history tracking)
  }, {
    isIdempotent: true,
    populateQS: (t, c, _, tableName) => {
      t.description = markdown` 
      This table contains the all the data/information about UDI-PGP excluding the suppliers.
     `;
      c.udi_pgp_config_id.description =
        `${tableName} primary key and internal label (not a ULID)`;
      c.addr.description = `the address the proxy server is bound to.`;
      c.metrics.description = `The address the health server started on.`;
      c.metrics.description = `The address the metrics server started on.`;
      c.config_ncl.description = `The most recent full NCL that was built and used for configuring the system.`
      c.admin_db_path.description = `The full path to admin state database for udi-pgp configuration and query logs.`
      c.surveilr_version.description = `The current version of surveilr that's being executed.`
      c.governance.description = `kernel-specific governance data`;
    },

    qualitySystem: {
      description: markdown`
            This table contains the all the data/information about UDI-PGP excluding the suppliers.
        `,
    },
  });

  const udiQueryObservabilty = gm.textPkTable("udi_pgp_observe_query_exec", {
    udi_pgp_observe_query_exec_id: gk.uuidPrimaryKey(),
    query_text: gd.text(),
    exec_start_at: gd.dateTime(),
    exec_finish_at: gd.dateTimeNullable(),
    elaboration: gd.jsonTextNullable(),
    exec_msg: gd.textArray,
    exec_status: gd.integer(),
    governance: gd.jsonTextNullable(),
    ...gm.housekeeping.columns, // activity_log should store previous versions in JSON format (for history tracking)
  }, {
    isIdempotent: true,
    populateQS: (t, c, _, tableName) => {
      t.description = markdown` 
      Get detailed information about the lifecycle of a query.
     `;
      c.udi_pgp_observe_query_exec_id.description =
        `${tableName} primary key and internal label (not a ULID)`;
      c.query_text.description = `Actual query string with comments included`;
      c.exec_start_at.description =
        `Timestamp for when the query execution began`;
      c.exec_finish_at.description =
        `Timestamp for when the query execution ended`;
      c.elaboration.description =
        `A general object containing details like all the events that transpired during the execution of a query`;
      c.exec_msg.description =
        `All errors encountered during the execution of a query`;
      c.exec_status.description =
        `An interger code denoting the execution statis of a query. A non zero code denotes that an error ocurred and thereby the "exec_msg" field should be populated`;
      c.governance.description = `kernel-specific governance data`;
    },

    qualitySystem: {
      description: markdown`
      Get detailed information about the lifecycle of a query.
        `,
    },
  });

  const informationSchema = {
    tables: [
      udiPgpSupplier,
      udiPgpConfig,
      udiQueryObservabilty,
      udiPgpSet,
    ],
    tableIndexes: [
      ...udiPgpSupplier.indexes,
      ...udiPgpConfig.indexes,
      ...udiQueryObservabilty.indexes,
      ...udiPgpSet.indexes,
    ],
  };

  return {
    modelsGovn,
    udiPgpSupplier,
    udiPgpConfig,
    udiQueryObservabilty,
    informationSchema,
  };
}
