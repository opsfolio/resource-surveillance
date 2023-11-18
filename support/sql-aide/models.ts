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
    assurance_schema_id: gk.textPrimaryKey(),
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

  const behavior = gm.textPkTable("behavior", {
    behavior_id: gm.keys.ulidPrimaryKey(),
    device_id: device.references.device_id(),
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
          application operations at runtime. For example FS Walk behaviors
          include configs that indicate which files to ignore, which to
          scan, when to load content, etc. This is more convenient than 
          creating 
          
          ${tableName} has a foreign key reference to the device table since
          behaviors might be device-specific.`;
      c.behavior_name.description =
        `Arbitrary but unique per-device behavior name (e.g. fs-walk::xyz)`;
      c.behavior_conf_json.description =
        `Configuration, settings, parameters, etc. describing the behavior (JSON, behavior-dependent)`;
      c.governance.description =
        `Descriptions or other "governance" details (JSON, behavior-dependent)`;
    },
  });

  const urWalkSession = gm.textPkTable("ur_walk_session", {
    ur_walk_session_id: gm.keys.ulidPrimaryKey(),
    device_id: device.references.device_id(),
    behavior_id: behavior.references.behavior_id().optional(),
    behavior_json: gd.jsonTextNullable(),
    walk_started_at: gd.dateTime(),
    walk_finished_at: gd.dateTimeNullable(),
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
        Immutable Walk Sessions represents any "discovery" or "walk" operation.
        This could be a device file system scan or any other resource discovery
        session. Each time a discovery operation starts, a record is created. 
        ${tableName} has a foreign key reference to the device table so that the
        same device can be used for multiple walk sessions but also the walk
        sessions can be merged across workstations / servers for easier detection
        of changes and similaries between file systems on different devices.`;
    },
  });

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
    populateQS: (t, _c, cols, _tableName) => {
      t.description = markdown`
        Immutable Walk Session path represents a discovery or "walk" path If
        the session was file system scan, then ${cols.root_path.identity} is the
        root file system path that was scanned. If the session was discovering
        resources in another target then ${cols.root_path.identity} would be
        representative of the target path (could be a URI).`;
    },
  });

  const uniformResource = gm.textPkTable(UNIFORM_RESOURCE, {
    uniform_resource_id: gm.keys.ulidPrimaryKey(),
    device_id: device.references.device_id(),
    walk_session_id: urWalkSession.references.ur_walk_session_id(),
    walk_path_id: urWalkSessionPath.references.ur_walk_session_path_id(),
    uri: gd.text(),
    content_digest: gd.text(),
    content: gd.blobTextNullable(),
    nature: gd.textNullable(),
    size_bytes: gd.integerNullable(),
    last_modified_at: gd.integerNullable(),
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
        key references to both ${urWalkSession.tableName} and ${urWalkSessionPath.tableName}
        tables to indicate which particular session and walk path the
        resourced was inserted during.`;
      c.uniform_resource_id.description = `${tableName} ULID primary key`;
      c.device_id.description =
        `which ${device.tableName} row introduced this resource`;
      c.walk_session_id.description =
        `which ${urWalkSession.tableName} row introduced this resource`;
      c.walk_path_id.description =
        `which ${urWalkSessionPath.tableName} row introduced this resource`;
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
      uniform_resource_transform_id: gm.keys.ulidPrimaryKey(),
      uniform_resource_id: uniformResource.references.uniform_resource_id(),
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

  const urWalkSessionPathFsEntry = gm.textPkTable(
    "ur_walk_session_path_fs_entry",
    {
      ur_walk_session_path_fs_entry_id: gm.keys.ulidPrimaryKey(),
      walk_session_id: urWalkSession.references
        .ur_walk_session_id(),
      walk_path_id: urWalkSessionPath.references.ur_walk_session_path_id(),
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
      behavior,
      urWalkSession,
      urWalkSessionPath,
      uniformResource,
      uniformResourceTransform,
      urWalkSessionPathFsEntry,
    ],
    tableIndexes: [
      ...device.indexes,
      ...behavior.indexes,
      ...urWalkSession.indexes,
      ...urWalkSessionPath.indexes,
      ...uniformResource.indexes,
      ...uniformResourceTransform.indexes,
      ...urWalkSessionPathFsEntry.indexes,
    ],
  };

  return {
    codeNbModels,
    device,
    behavior,
    urWalkSession,
    urWalkSessionPath,
    uniformResource,
    uniformResourceTransform,
    urWalkSessionPathFsEntry,
    informationSchema,
  };
}
