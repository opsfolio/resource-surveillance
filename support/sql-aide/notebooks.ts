import { chainNB, SQLa, SQLa_tp as typical, tbls } from "./deps.ts";
import * as m from "./models.ts";

// deno-lint-ignore no-explicit-any
type Any = any;

// TODO: https://github.com/opsfolio/resource-surveillance/issues/17
//       Integrate SQLa Quality System functionality so that documentation
//       is not just in code but makes its way into the database.

// ServiceContentHelpers creates "insertable" type-safe content objects needed by the other notebooks
// SqlNotebookHelpers encapsulates instances of SQLa objects needed to creation of SQL text in the other notebooks
// BootstrapSqlNotebook encapsulates DDL and table/view/entity for starting a database from scratch
// ConstructionSqlNotebook encapsulates DDL and table/view/entity construction (what are called stateful "migrations" in other systems, should be assumed to be non-idempotent)
// MutationSqlNotebook encapsulates DML and stateful table data insert/update/delete
// QuerySqlNotebook encapsulates DQL and stateless table queries that can operate all within SQLite
// AssuranceSqlNotebook encapsulates DQL and stateless TAP-formatted test cases
// SQLPageNotebook encapsulates [SQLPage](https://sql.ophir.dev/) content
// SqlNotebooksOrchestrator encapsulates instances of all the other notebooks and provides performs all the work

// Reminders:
// - when sending arbitrary text to the SQL stream, use SqlTextBehaviorSupplier
// - when sending SQL statements (which need to be ; terminated) use SqlTextSupplier
// - use jtladeiras.vscode-inline-sql, frigus02.vscode-sql-tagged-template-literals-syntax-only or similar SQL syntax highlighters in VS Code so it's easier to edit SQL

/**
 * MORE TODO for README.md:
 * Our SQL "notebook" is a library function which is responsible to pulling
 * together all SQL we use. It's important to note we do not prefer to use ORMs
 * that hide SQL and instead use stateless SQL generators like SQLa to produce
 * all SQL through type-safe TypeScript functions.
 *
 * Because applications come and go but data lives forever, we want to allow
 * our generated SQL to be hand-edited later if the initial generated code no
 * longers benefits from being regenerated in the future.
 *
 * We go to great lengths to allow SQL to be independently executed because we
 * don't always know the final use cases and we try to use the SQLite CLI whenever
 * possible because performance is best that way.
 *
 * Because SQL is a declarative and TypeScript is imperative langauage, use each
 * for their respective strengths. Use TypeScript to generate type-safe SQL and
 * let the database do as much work as well.
 * - Capture all state, valid content, invalid content, and other data in the
 *   database so that we can run queries for observability; if everything is in
 *   the database, including error messages, warnings, etc. we can always run
 *   queries and not have to store logs in separate system.
 * - Instead of imperatively creating thousands of SQL statements, let the SQL
 *   engine use CTEs and other capabilities to do as much declarative work in
 *   the engine as possible.
 * - Instead of copy/pasting SQL into multiple SQL statements, modularize the
 *   SQL in TypeScript functions and build statements using template literal
 *   strings (`xyz${abc}`).
 * - Wrap SQL into TypeScript as much as possible so that SQL statements can be
 *   pulled in from URLs.
 * - If we're importing JSON, CSV, or other files pull them in via
 *   `import .. from "xyz" with { type: "json" }` and similar imports in case
 *   the SQL engine cannot do the imports directly from URLs (e.g. DuckDB can
 *   import HTTP directly and should do so, SQLite can pull from URLs too with
 *   the http0 extension).
 * - Whenever possible make SQL stateful functions like DDL, DML, etc. idempotent
 *   either by using `ON CONFLICT DO NOTHING` or when a conflict occurs put the
 *   errors or warnings into a table that the application should query.
 */

function codeBlock(
  literals: TemplateStringsArray,
  ...expressions: unknown[]
): string {
  // Remove the first line if it has whitespace only or is a blank line
  const firstLiteral = literals[0].replace(/^(\s+\n|\n)/, "");
  const indentation = /^(\s*)/.exec(firstLiteral);

  let result: string;
  if (indentation) {
    const replacer = new RegExp(`^${indentation[1]}`, "gm");
    result = firstLiteral.replaceAll(replacer, "");
    for (let i = 0; i < expressions.length; i++) {
      result += expressions[i] + literals[i + 1].replaceAll(replacer, "");
    }
  } else {
    result = firstLiteral;
    for (let i = 0; i < expressions.length; i++) {
      result += expressions[i] + literals[i + 1];
    }
  }

  return result;
}

/**
 * Decorate a function with `@notIdempotent` if it's important to indicate
 * whether its SQL is idempotent or not. By default we assume all SQL is
 * idempotent but this can be set to indicate it's not.
 */
export const notIdempotent = <Notebook>(
  cells: Set<chainNB.NotebookCellID<Notebook>>,
) => {
  return (
    _target: SQLa.SqlNotebook<Any>,
    propertyKey: chainNB.NotebookCellID<Notebook>,
    _descriptor: PropertyDescriptor,
  ) => {
    cells.add(propertyKey);
  };
};

/**
 * Decorate a function with `@dontStoreInDB` if the particular query should
 * not be stored in the code_notebook_cell table in the database.
 */
export const dontStoreInDB = <Notebook>(
  cells: Set<chainNB.NotebookCellID<Notebook>>,
) => {
  return (
    _target: SQLa.SqlNotebook<Any>,
    propertyKey: chainNB.NotebookCellID<Notebook>,
    _descriptor: PropertyDescriptor,
  ) => {
    cells.add(propertyKey);
  };
};

export const noSqliteExtnLoader: (
  extn: string,
) => SQLa.SqlTextBehaviorSupplier<Any> = (extn: string) => ({
  executeSqlBehavior: () => ({
    SQL: () => `-- loadExtnSQL not provided to load '${extn}'`,
  }),
});

async function gitLikeHash(content: string) {
  // Git header for a blob object (change 'blob' to 'commit' or 'tree' for those objects)
  // This assumes the content is plain text, so we can get its length as a string
  const header = `blob ${content.length}\0`;

  // Combine header and content
  const combinedContent = new TextEncoder().encode(header + content);

  // Compute SHA-1 hash
  const hashBuffer = await crypto.subtle.digest("SHA-1", combinedContent);

  // Convert hash to hexadecimal string
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map((b) => b.toString(16).padStart(2, "0")).join(
    "",
  );

  return hashHex;
}

/**
 * Encapsulates instances of SQLa objects needed to creation of SQL text in the
 * other notebooks. An instance of this class is usually passed into all the
 * other notebooks.
 */
export class SqlNotebookHelpers<EmitContext extends SQLa.SqlEmitContext>
  extends SQLa.SqlNotebook<EmitContext> {
  readonly emitCtx: EmitContext;
  readonly models: ReturnType<typeof m.serviceModels<EmitContext>>;
  readonly loadExtnSQL: (
    extn: string,
  ) => SQLa.SqlTextBehaviorSupplier<EmitContext>;
  readonly stsOptions: SQLa.SqlTextSupplierOptions<EmitContext>;
  readonly modelsGovn: ReturnType<
    typeof m.serviceModels<EmitContext>
  >["codeNbModels"]["modelsGovn"];
  readonly templateState: ReturnType<
    typeof m.serviceModels<EmitContext>
  >["codeNbModels"]["modelsGovn"]["templateState"];

  constructor(
    readonly options?: {
      readonly loadExtnSQL?: (
        extn: string,
      ) => SQLa.SqlTextBehaviorSupplier<EmitContext>;
      readonly models?: ReturnType<typeof m.serviceModels<EmitContext>>;
      readonly stsOptions?: SQLa.SqlTextSupplierOptions<EmitContext>;
    },
  ) {
    super();
    this.models = options?.models ?? m.serviceModels<EmitContext>();
    this.modelsGovn = this.models.codeNbModels.modelsGovn;
    this.emitCtx = this.modelsGovn.sqlEmitContext();
    this.templateState = this.modelsGovn.templateState;
    this.loadExtnSQL = options?.loadExtnSQL ?? noSqliteExtnLoader;
    this.stsOptions = options?.stsOptions ??
      SQLa.typicalSqlTextSupplierOptions();
  }

  // type-safe wrapper for all SQL text generated in this library;
  // we call it `SQL` so that VS code extensions like frigus02.vscode-sql-tagged-template-literals
  // properly syntax-highlight code inside SQL`xyz` strings.
  get SQL() {
    return SQLa.SQL<EmitContext>(this.templateState.ddlOptions);
  }

  renderSqlCmd() {
    return SQLa.RenderSqlCommand.renderSQL<EmitContext>((sts) =>
      sts.SQL(this.emitCtx)
    );
  }

  // type-safe wrapper for all SQL that should not be treated as SQL statements
  // but as arbitrary text to send to the SQL stream
  sqlBehavior(
    sts: SQLa.SqlTextSupplier<EmitContext>,
  ): SQLa.SqlTextBehaviorSupplier<EmitContext> {
    return {
      executeSqlBehavior: () => sts,
    };
  }

  // ULID generator when the value is needed by the SQLite engine runtime
  get sqlEngineNewUlid(): SQLa.SqlTextSupplier<EmitContext> {
    return { SQL: () => `ulid()` };
  }

  get onConflictDoNothing(): SQLa.SqlTextSupplier<EmitContext> {
    return { SQL: () => `ON CONFLICT DO NOTHING` };
  }

  // ULID generator when the value is needed by the SQLite engine runtime
  get sqlEngineNow(): SQLa.SqlTextSupplier<EmitContext> {
    return { SQL: () => `CURRENT_TIMESTAMP` };
  }

  /**
   * Setup the SQL bind parameters; object property values will be available as
   * :key1, :key2, etc.
   * @param shape is an object with key value pairs that we want to convert to SQLite parameters
   * @returns the rewritten object (using new keys) and the associated DML
   */
  sqlParameters<
    Shape extends Record<
      string,
      string | number | SQLa.SqlTextSupplier<EmitContext>
    >,
  >(shape: Shape) {
    /**
     * This is a "virtual" table that should not be used for DDL but used for DML.
     * It is managed by SQLite and is used to store `.parameter set` values and
     * allows all keys to be used as `:xyz` variables that point to `value`.
     *
     * SQLite shell `.parameter set xyz value` is equivalent to `INSERT INTO
     * sqlite_parameters (key, value) VALUES ('xyz', 'value')` but `.parameter set`
     * does not support SQL expressions. If you need a value to be evaluated before
     * being set then use `INSERT INTO sqlite_parameters (key, value)...`.
     */
    const { model: gm, domains: gd } = this.modelsGovn;
    const sqp = gm.table("sqlite_parameters", {
      key: gd.text(),
      value: gd.text(),
    });

    const paramsDML = Object.entries(shape).map(([key, value]) =>
      sqp.insertDML({
        key: `:${key}`,
        value: typeof value === "number" ? String(value) : value,
      })
    );

    type SqlParameters = { [K in keyof Shape as `:${string & K}`]: Shape[K] };
    return {
      params: (): SqlParameters => {
        const newShape: Partial<SqlParameters> = {};
        for (const key in shape) {
          const newKey = `:${key}`;
          (newShape as Any)[newKey] = shape[key];
        }
        return newShape as unknown as SqlParameters;
      },
      paramsDML,
    };
  }

  viewDefn<ViewName extends string, DomainQS extends SQLa.SqlDomainQS>(
    viewName: ViewName,
  ) {
    return SQLa.viewDefinition<ViewName, EmitContext, DomainQS>(viewName, {
      isIdempotent: true,
      embeddedStsOptions: this.templateState.ddlOptions,
      before: (viewName) => SQLa.dropView(viewName),
    });
  }
}

export type KernelID = "SQL" | "PlantUML" | "LLM Prompt";

/**
 * Encapsulates SQL DDL and table/view/entity construction SQLa objects for
 * "bootstrapping" (creating a SQLite database from scratch). The actual models
 * are not managed by this class but it does include all the migration scripts
 * which assemble the other SQL into a migration steps.
 */
export class BootstrapSqlNotebook<
  EmitContext extends SQLa.SqlEmitContext,
> extends SQLa.SqlNotebook<EmitContext> {
  constructor(readonly nbh: SqlNotebookHelpers<EmitContext>) {
    super();
  }

  bootstrapDDL() {
    const { nbh, nbh: { models: { codeNbModels } } } = this;
    // deno-fmt-ignore
    return nbh.SQL`
      ${codeNbModels.informationSchema.tables}

      ${codeNbModels.informationSchema.tableIndexes}
      `;
  }

  bootstrapSeedDML() {
    const {
      nbh,
      nbh: { models: { codeNbModels: { codeNotebookKernel: kernel } } },
    } = this;
    const created_at = nbh.sqlEngineNow;
    const options = {
      onConflict: {
        SQL: () =>
          `ON CONFLICT(kernel_name) DO UPDATE SET mime_type = EXCLUDED.mime_type, file_extn = EXCLUDED.file_extn`,
      },
    };
    const sql = kernel.insertDML({
      code_notebook_kernel_id: "SQL",
      kernel_name: "Dialect-independent ANSI SQL",
      mime_type: "application/sql",
      file_extn: ".sql",
      created_at,
    }, options);
    const denoTaskShell = kernel.insertDML({
      code_notebook_kernel_id: "DenoTaskShell",
      kernel_name: "Deno Task Shell",
      mime_type: "application/x-deno-task-sh",
      file_extn: ".deno-task-sh",
      created_at,
    }, options);
    const puml = kernel.insertDML({
      code_notebook_kernel_id: "PlantUML",
      kernel_name: "PlantUML ER Diagram",
      mime_type: "text/vnd.plantuml",
      file_extn: ".puml",
      created_at,
    }, options);
    const llmPrompt = kernel.insertDML({
      code_notebook_kernel_id: "LLM Prompt",
      kernel_name: "Large Lanugage Model (LLM) Prompt",
      mime_type: "text/vnd.netspective.llm-prompt",
      file_extn: ".llm-prompt.txt",
      created_at,
    }, options);
    return [sql, denoTaskShell, puml, llmPrompt];
  }
}

/**
 * Encapsulates SQL DDL and table/view/entity construction SQLa objects. The
 * actual models are not managed by this class but it does include all the
 * migration scripts which assemble the other SQL into a migration steps.
 *
 * Cell name format: {version}_{pragma}_{abitrary}
 * - {version} is used for sorting on `select`
 * - {pragma} may be:
 *   - `once_` to only run the script if its contents have not been run before
 *     (see https://www.chezmoi.io/reference/source-state-attributes/ for ideas)
 * - {arbitrary} may be anything else
 */
export class ConstructionSqlNotebook<EmitContext extends SQLa.SqlEmitContext>
  extends SQLa.SqlNotebook<EmitContext> {
  constructor(
    readonly nbh: SqlNotebookHelpers<EmitContext>,
    readonly storedNotebookStateTransitions: ReturnType<
      typeof nbh.models.codeNbModels.codeNotebookState.select
    >["filterable"][],
  ) {
    super();
  }

  // note `once_` pragma means it must only be run once in the database
  v001_once_initialDDL() {
    const { nbh, nbh: { models } } = this;
    // deno-fmt-ignore
    return nbh.SQL`
      ${models.informationSchema.tables}

      ${models.informationSchema.tableIndexes}
      `;
  }

  // note since `once_` pragma is not present, it will be run each time
  v002_fsContentIngestSessionStatsViewDDL() {
    // deno-fmt-ignore
    return this.nbh.viewDefn("ingest_session_stats")/* sql */`
      WITH Summary AS (
          SELECT
              device.device_id AS device_id,
              ur_ingest_session.ur_ingest_session_id AS ingest_session_id,
              ur_ingest_session.ingest_started_at AS ingest_session_started_at,
              ur_ingest_session.ingest_finished_at AS ingest_session_finished_at,
              COALESCE(ur_ingest_session_fs_path_entry.file_extn, '') AS file_extension,
              ur_ingest_session_fs_path.ur_ingest_session_fs_path_id as ingest_session_fs_path_id,
              ur_ingest_session_fs_path.root_path AS ingest_session_root_fs_path,
              COUNT(ur_ingest_session_fs_path_entry.uniform_resource_id) AS total_file_count,
              SUM(CASE WHEN uniform_resource.content IS NOT NULL THEN 1 ELSE 0 END) AS file_count_with_content,
              SUM(CASE WHEN uniform_resource.frontmatter IS NOT NULL THEN 1 ELSE 0 END) AS file_count_with_frontmatter,
              MIN(uniform_resource.size_bytes) AS min_file_size_bytes,
              AVG(uniform_resource.size_bytes) AS average_file_size_bytes,
              MAX(uniform_resource.size_bytes) AS max_file_size_bytes,
              MIN(uniform_resource.last_modified_at) AS oldest_file_last_modified_datetime,
              MAX(uniform_resource.last_modified_at) AS youngest_file_last_modified_datetime
          FROM
              ur_ingest_session
          JOIN
              device ON ur_ingest_session.device_id = device.device_id
          LEFT JOIN
              ur_ingest_session_fs_path ON ur_ingest_session.ur_ingest_session_id = ur_ingest_session_fs_path.ingest_session_id
          LEFT JOIN
              ur_ingest_session_fs_path_entry ON ur_ingest_session_fs_path.ur_ingest_session_fs_path_id = ur_ingest_session_fs_path_entry.ingest_fs_path_id
          LEFT JOIN
              uniform_resource ON ur_ingest_session_fs_path_entry.uniform_resource_id = uniform_resource.uniform_resource_id
          GROUP BY
              device.device_id,
              ur_ingest_session.ur_ingest_session_id,
              ur_ingest_session.ingest_started_at,
              ur_ingest_session.ingest_finished_at,
              ur_ingest_session_fs_path_entry.file_extn,
              ur_ingest_session_fs_path.root_path
      )
      SELECT
          device_id,
          ingest_session_id,
          ingest_session_started_at,
          ingest_session_finished_at,
          file_extension,
          ingest_session_fs_path_id,
          ingest_session_root_fs_path,
          total_file_count,
          file_count_with_content,
          file_count_with_frontmatter,
          min_file_size_bytes,
          CAST(ROUND(average_file_size_bytes) AS INTEGER) AS average_file_size_bytes,
          max_file_size_bytes,
          oldest_file_last_modified_datetime,
          youngest_file_last_modified_datetime
      FROM
          Summary
      ORDER BY
          device_id,
          ingest_session_finished_at,
          file_extension;
      `;
  }

  // note since `once_` pragma is not present, it will be run each time
  v002_fsContentIngestSessionStatsLatestViewDDL() {
    // deno-fmt-ignore
    return this.nbh.viewDefn("ingest_session_stats_latest")/* sql */`
      SELECT iss.*
        FROM ingest_session_stats AS iss
        JOIN (  SELECT ur_ingest_session.ur_ingest_session_id AS latest_session_id
                  FROM ur_ingest_session
              ORDER BY ur_ingest_session.ingest_finished_at DESC
                 LIMIT 1) AS latest
          ON iss.ingest_session_id = latest.latest_session_id;`;
  }

  v002_urIngestSessionIssueViewDDL() {
    // deno-fmt-ignore
    return this.nbh.viewDefn("ur_ingest_session_issue")/* sql */`
        SELECT us.device_id,
               us.ur_ingest_session_id,
               usp.ur_ingest_session_fs_path_id,
               usp.root_path,
               ufs.ur_ingest_session_fs_path_entry_id,
               ufs.file_path_abs,
               ufs.ur_status,
               ufs.ur_diagnostics
          FROM ur_ingest_session_fs_path_entry ufs
          JOIN ur_ingest_session_fs_path usp ON ufs.ingest_fs_path_id = usp.ur_ingest_session_fs_path_id
          JOIN ur_ingest_session us ON usp.ingest_session_id = us.ur_ingest_session_id
         WHERE ufs.ur_status IS NOT NULL
      GROUP BY us.device_id,
               us.ur_ingest_session_id,
               usp.ur_ingest_session_fs_path_id,
               usp.root_path,
               ufs.ur_ingest_session_fs_path_entry_id,
               ufs.file_path_abs,
               ufs.ur_status,
               ufs.ur_diagnostics;`
  }
}

/**
 * Encapsulates SQL DML and stateful table data insert/update/delete operations.
 */
export class MutationSqlNotebook<EmitContext extends SQLa.SqlEmitContext>
  extends SQLa.SqlNotebook<EmitContext> {
  constructor(readonly nbh: SqlNotebookHelpers<EmitContext>) {
    super();
  }
}

/**
 * Encapsulates SQL DQL and stateless table queries that can operate all within
 * SQLite (means they are "storable" in code_notebook_cell table).
 */
export class QuerySqlNotebook<EmitContext extends SQLa.SqlEmitContext>
  extends SQLa.SqlNotebook<EmitContext> {
  constructor(readonly nbh: SqlNotebookHelpers<EmitContext>) {
    super();
  }

  /*
   * This SQL statement retrieves column information for tables in an SQLite database
   * including table name, column ID, column name, data type, nullability, default
   * value, and primary key status.
   * It filters only tables from the result set. It is commonly used for analyzing
   * and documenting database schemas.
   * NOTE: pragma_table_info(m.tbl_name) will only work when m.type is 'table'
   * TODO: add all the same content that is emitted by infoSchemaMarkdown
   */
  infoSchema() {
    return this.nbh.SQL`
      SELECT tbl_name AS table_name,
             c.cid AS column_id,
             c.name AS column_name,
             c."type" AS "type",
             c."notnull" AS "notnull",
             c.dflt_value as "default_value",
             c.pk AS primary_key
        FROM sqlite_master m,
             pragma_table_info(m.tbl_name) c
       WHERE m.type = 'table';`;
  }

  /**
   * Generates a JSON configuration for osquery's auto_table_construction
   * feature by inspecting the SQLite database schema. The SQL creates a
   * structured JSON object detailing each table within the database. For
   * every table, the object includes a standard SELECT query, the relevant
   * columns, and the database file path.
   *
   * @example
   * // The resultant JSON object is structured as follows:
   * {
   *   "auto_table_construction": {
   *     "table_name1": {
   *       "query": "SELECT column1, column2, ... FROM table_name1",
   *       "columns": ["column1", "column2", ...],
   *       "path": "./sqlite-src.db"
   *     },
   *     ...
   *   }
   * }
   */
  infoSchemaOsQueryATCs() {
    return this.nbh.SQL`
      WITH table_columns AS (
          SELECT m.tbl_name AS table_name,
                 group_concat(c.name) AS column_names_for_select,
                 json_group_array(c.name) AS column_names_for_atc_json
            FROM sqlite_master m,
                 pragma_table_info(m.tbl_name) c
           WHERE m.type = 'table'
        GROUP BY m.tbl_name
      ),
      target AS (
        -- set SQLite parameter :osquery_atc_path to assign a different path
        SELECT COALESCE(:osquery_atc_path, 'SQLITEDB_PATH') AS path
      ),
      table_query AS (
          SELECT table_name,
                 'SELECT ' || column_names_for_select || ' FROM ' || table_name AS query,
                 column_names_for_atc_json
            FROM table_columns
      )
      SELECT json_object('auto_table_construction',
                json_group_object(
                    table_name,
                    json_object(
                        'query', query,
                        'columns', json(column_names_for_atc_json),
                        'path', path
                    )
                )
             ) AS osquery_auto_table_construction
        FROM table_query, target;`;
  }

  /**
   * SQL which generates the Markdown content lines (rows) which describes all
   * the tables, columns, indexes, and views in the database. This should really
   * be a view instead of a query but SQLite does not support use of pragma_* in
   * views for security reasons.
   * TODO: check out https://github.com/k1LoW/tbls and make this query equivalent
   *       to that utility's output including generating PlantUML through SQL.
   */
  infoSchemaMarkdown() {
    return this.nbh.SQL`
      -- TODO: https://github.com/lovasoa/SQLpage/discussions/109#discussioncomment-7359513
      --       see the above for how to fix for SQLPage but figure out to use the same SQL
      --       in and out of SQLPage (maybe do what Ophir said in discussion and create
      --       custom output for SQLPage using componetns?)
      WITH TableInfo AS (
        SELECT
          m.tbl_name AS table_name,
          CASE WHEN c.pk THEN '*' ELSE '' END AS is_primary_key,
          c.name AS column_name,
          c."type" AS column_type,
          CASE WHEN c."notnull" THEN '*' ELSE '' END AS not_null,
          COALESCE(c.dflt_value, '') AS default_value,
          COALESCE((SELECT pfkl."table" || '.' || pfkl."to" FROM pragma_foreign_key_list(m.tbl_name) AS pfkl WHERE pfkl."from" = c.name), '') as fk_refs,
          ROW_NUMBER() OVER (PARTITION BY m.tbl_name ORDER BY c.cid) AS row_num
        FROM sqlite_master m JOIN pragma_table_info(m.tbl_name) c ON 1=1
        WHERE m.type = 'table'
        ORDER BY table_name, row_num
      ),
      Views AS (
        SELECT '## Views ' AS markdown_output
        UNION ALL
        SELECT '| View | Column | Type |' AS markdown_output
        UNION ALL
        SELECT '| ---- | ------ |----- |' AS markdown_output
        UNION ALL
        SELECT '| ' || tbl_name || ' | ' || c.name || ' | ' || c."type" || ' | '
        FROM
          sqlite_master m,
          pragma_table_info(m.tbl_name) c
        WHERE
          m.type = 'view'
      ),
      Indexes AS (
        SELECT '## Indexes' AS markdown_output
        UNION ALL
        SELECT '| Table | Index | Columns |' AS markdown_output
        UNION ALL
        SELECT '| ----- | ----- | ------- |' AS markdown_output
        UNION ALL
        SELECT '| ' ||  m.name || ' | ' || il.name || ' | ' || group_concat(ii.name, ', ') || ' |' AS markdown_output
        FROM sqlite_master as m,
          pragma_index_list(m.name) AS il,
          pragma_index_info(il.name) AS ii
        WHERE
          m.type = 'table'
        GROUP BY
          m.name,
          il.name
      )
      SELECT
          markdown_output AS info_schema_markdown
      FROM
        (
          SELECT '## Tables' AS markdown_output
          UNION ALL
          SELECT
            CASE WHEN ti.row_num = 1 THEN '
      ### \`' || ti.table_name || '\` Table
      | PK | Column | Type | Req? | Default | References |
      | -- | ------ | ---- | ---- | ------- | ---------- |
      ' ||
              '| ' || is_primary_key || ' | ' || ti.column_name || ' | ' || ti.column_type || ' | ' || ti.not_null || ' | ' || ti.default_value || ' | ' || ti.fk_refs || ' |'
            ELSE
              '| ' || is_primary_key || ' | ' || ti.column_name || ' | ' || ti.column_type || ' | ' || ti.not_null || ' | ' || ti.default_value || ' | ' || ti.fk_refs || ' |'
            END
          FROM TableInfo ti
          UNION ALL SELECT ''
          UNION ALL SELECT * FROM	Views
          UNION ALL SELECT ''
          UNION ALL SELECT * FROM Indexes
      );`;
  }

  htmlAnchors() {
    // deno-fmt-ignore
    return this.nbh.SQL`
        ${this.nbh.loadExtnSQL("asg017/html/html0")}

        -- find all HTML files in the uniform_resource table and return
        -- each file and the anchors' labels and hrefs in that file
        -- TODO: create a table called fs_content_html_anchor to store this data after inserting it into uniform_resource
        --       so that simple HTML lookups do not require the html0 extension to be loaded
        WITH html_content AS (
          SELECT uniform_resource_id, content, content_digest, file_path, file_extn FROM uniform_resource WHERE nature = 'html'
        ),
        html AS (
          SELECT file_path,
                 text as label,
                 html_attribute_get(html, 'a', 'href') as href
            FROM html_content, html_each(html_content.content, 'a')
        )
        SELECT * FROM html;
      `;
  }

  htmlHeadMeta() {
    // deno-fmt-ignore
    return this.nbh.SQL`
        ${this.nbh.loadExtnSQL("asg017/html/html0")}

        -- find all HTML files in the uniform_resource table and return
        -- each file and the <head><meta name="key" content="value"> pair
        -- TODO: create a table called resource_html_head_meta to store this data after inserting it into uniform_resource
        --       so that simple HTML lookups do not require the html0 extension to be loaded
        WITH html_content AS (
          SELECT uniform_resource_id, content, content_digest, file_path, file_extn FROM uniform_resource WHERE nature = 'html'
        ),
        html AS (
          SELECT file_path,
                 html_attribute_get(html, 'meta', 'name') as key,
                 html_attribute_get(html, 'meta', 'content') as value,
                 html
            FROM html_content, html_each(html_content.content, 'head meta')
           WHERE key IS NOT NULL
        )
        SELECT * FROM html;
      `;
  }
}

/**
 * Encapsulates [SQLPage](https://sql.ophir.dev/) content. SqlPageNotebook has
 * methods with the name of each [SQLPage](https://sql.ophir.dev/) content that
 * we want in the database. The MutationSqlNotebook sqlPageSeedDML method
 * "reads" the cells in the SqlPageNotebook (each method's result) and
 * generates SQL to insert the content of the page in the database in the format
 * and table expected by [SQLPage](https://sql.ophir.dev/).
 * NOTE: we break our PascalCase convention for the name of the class since SQLPage
 *       is a proper noun (product name).
 */
export class SQLPageNotebook<EmitContext extends SQLa.SqlEmitContext>
  extends SQLa.SqlNotebook<EmitContext> {
  // if you want to add any annotations, use this like:
  //   @SQLPageNotebook.nbd.init(), .finalize(), etc.
  //   @SQLPageNotebook.nbd.disregard(), etc.
  static nbd = new chainNB.NotebookDescriptor<
    SQLPageNotebook<Any>,
    chainNB.NotebookCell<
      SQLPageNotebook<Any>,
      chainNB.NotebookCellID<SQLPageNotebook<Any>>
    >
  >();
  readonly queryNB: QuerySqlNotebook<EmitContext>;

  constructor(readonly nbh: SqlNotebookHelpers<EmitContext>) {
    super();
    this.queryNB = new QuerySqlNotebook(this.nbh);
  }

  "index.sql"() {
    return this.nbh.SQL`
      SELECT
        'list' as component,
        'Get started: where to go from here ?' as title,
        'Here are some useful links to get you started with SQLPage.' as description;
      SELECT 'Content Ingestion Session Statistics' as title,
        'ingest-session-stats.sql' as link,
        'TODO' as description,
        'green' as color,
        'download' as icon;
      SELECT 'MIME Types' as title,
        'mime-types.sql' as link,
        'TODO' as description,
        'blue' as color,
        'download' as icon;
      SELECT 'Stored SQL Notebooks' as title,
        'notebooks.sql' as link,
        'TODO' as description,
        'blue' as color,
        'download' as icon;
      SELECT 'Information Schema' as title,
        'info-schema.sql' as link,
        'TODO' as description,
        'blue' as color,
        'download' as icon;`;
  }

  "ingest-session-stats.sql"() {
    return this.nbh.SQL`
      SELECT 'table' as component, 1 as search, 1 as sort;
      SELECT ingest_session_started_at, file_extn, total_count, with_content, with_frontmatter, average_size from ingest_session_stats;`;
  }

  "mime-types.sql"() {
    return this.nbh.SQL`
      SELECT 'table' as component, 1 as search, 1 as sort;
      SELECT name, file_extn, description from mime_type;`;
  }

  "notebooks.sql"() {
    const { codeNbModels: { codeNotebookCell: cnbc } } = this.nbh.models;
    const { symbol: scnbc } = cnbc.columnNames(this.nbh.emitCtx);

    return this.nbh.SQL`
      SELECT 'table' as component, 'Cell' as markdown, 1 as search, 1 as sort;
      SELECT ${scnbc.notebook_name},
             '[' || ${scnbc.cell_name} || '](notebook-cell.sql?notebook=' ||  ${scnbc.notebook_name} || '&cell=' || ${scnbc.cell_name} || ')' as Cell
        FROM ${cnbc.tableName};`;
  }

  "notebook-cell.sql"() {
    const { codeNbModels: { codeNotebookCell: cnbc } } = this.nbh.models;
    const { symbol: scnbc } = cnbc.columnNames(this.nbh.emitCtx);

    return this.nbh.SQL`
      SELECT 'text' as component,
             $notebook || '.' || $cell as title,
             '\`\`\`sql
      ' || ${scnbc.interpretable_code} || '
      \`\`\`' as contents_md
       FROM ${cnbc.tableName}
      WHERE ${scnbc.notebook_name} = $notebook
        AND ${scnbc.cell_name} = $cell;`;
  }

  "info-schema.sql"() {
    return this.nbh.SQL`
      ${this.queryNB.infoSchemaMarkdown()}

      -- :info_schema_markdown should be defined in the above query
      SELECT 'text' as component,
             'Information Schema' as title,
             :info_schema_markdown as contents_md`;
  }

  "bad-item.sql"() {
    return "this is not a proper return type in SQLPageNotebook so it should generate an alert page in SQLPage (included just for testing)";
  }

  @SQLPageNotebook.nbd.disregard()
  "disregarded.sql"() {
    return "this should be disregarded and not included in SQLPage (might be a support function)";
  }

  // TODO: add one or more pages that will contain PlantUML or database
  //       description markdown so that the documentation for the database
  //       is contained within the DB itself.

  static create<EmitContext extends SQLa.SqlEmitContext>(
    nbh: SqlNotebookHelpers<EmitContext>,
  ) {
    const kernel = chainNB.ObservableKernel.create(
      SQLPageNotebook.prototype,
      SQLPageNotebook.nbd,
    );
    const instance = new SQLPageNotebook(nbh);
    return {
      kernel,
      instance,
      SQL: async () => {
        const irs = await kernel.initRunState();
        const { model: gm, domains: gd, keys: gk } = nbh.modelsGovn;
        const sqlPageFiles = gm.table("sqlpage_files", {
          path: gk.varcharPrimaryKey(),
          contents: gd.text(),
          last_modified: gd.createdAt(),
        }, {
          isIdempotent: true,
          qualitySystem: {
            description: m.markdown`
              [SQLPage](https://sql.ophir.dev/) app server content`,
          },
        });
        const ctx = nbh.emitCtx;
        const seedSQL: SQLa.SqlTextSupplier<EmitContext>[] = [sqlPageFiles];
        irs.runState.eventEmitter.afterCell = (cell, state) => {
          if (state.status == "successful") {
            seedSQL.push(sqlPageFiles.insertDML({
              path: cell, // the class's method name is the "cell"
              // deno-fmt-ignore
              contents: SQLa.isSqlTextSupplier<EmitContext>(state.execResult)
                ? state.execResult.SQL(ctx)
                : `select 'alert' as component,
                            'MutationSqlNotebook.SQLPageSeedDML() issue' as title,
                            'SQLPageNotebook cell "${cell}" did not return SQL (found: ${typeof state.execResult})' as description;`,
              last_modified: nbh.sqlEngineNow,
            }, {
              onConflict: {
                SQL: () =>
                  `ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP`,
              },
            }));
          }
        };

        await kernel.run(instance, irs);
        return seedSQL;
      },
    };
  }
}

export class AssuranceSqlNotebook<EmitContext extends SQLa.SqlEmitContext>
  extends SQLa.SqlNotebook<EmitContext> {
  readonly queryNB: QuerySqlNotebook<EmitContext>;

  constructor(readonly nbh: SqlNotebookHelpers<EmitContext>) {
    super();
    this.queryNB = new QuerySqlNotebook(this.nbh);
  }

  test1() {
    return this.nbh.SQL`
      WITH test_plan AS (
          SELECT '1..1' AS tap_output
      ),
      test1 AS (  -- Check if the 'fileio' extension is loaded by calling the 'readfile' function
          SELECT
              CASE
                  WHEN readfile('README.md') IS NOT NULL THEN 'ok 1 - fileio extension is loaded.'
                  ELSE 'not ok 1 - fileio extension is not loaded.'
              END AS tap_output
          FROM (SELECT 1) -- This is a dummy table of one row to ensure the SELECT runs.
      )
      SELECT tap_output FROM test_plan
      UNION ALL
      SELECT tap_output FROM test1;`;
  }
}

/**
 * Chain-of-Responsiblity style notebook base class
 */
export abstract class CodeNotebook<Context extends SQLa.SqlEmitContext> {
}

export function codeNotebookAnnotations<
  Notebook extends CodeNotebook<Context>,
  Context extends SQLa.SqlEmitContext,
>() {
  return new chainNB.NotebookDescriptor<
    Notebook,
    chainNB.NotebookCell<Notebook, chainNB.NotebookCellID<Notebook>>
  >();
}

export function codeNotebookFactory<
  Notebook extends CodeNotebook<EmitContext>,
  EmitContext extends SQLa.SqlEmitContext,
>(
  prototype: Notebook,
  instance: () => Notebook,
  nbd = codeNotebookAnnotations<Notebook, EmitContext>(),
) {
  type CellID = chainNB.NotebookCellID<Notebook>;
  const kernel = chainNB.ObservableKernel.create(prototype, nbd);

  type EventEmitter = Awaited<
    ReturnType<typeof kernel.initRunState>
  >["runState"]["eventEmitter"];
  return {
    nbd,
    kernel,
    instance,
    cellsDML: async (
      nbh: SqlNotebookHelpers<EmitContext>,
      kernelID: KernelID,
      notebookName: string,
    ) => {
      const { codeNbModels: { codeNotebookCell } } = nbh.models;
      const sqlDML: SQLa.SqlTextSupplier<EmitContext>[] = [];
      // prepare the run state with list of all pages defined and have the kernel
      // traverse the cells and emit (the SQL generator, no SQL is executed)
      const irs = await kernel.initRunState();
      irs.runState.eventEmitter.afterCell = async (cell, state) => {
        if (state.status == "successful") {
          const interpretable_code = typeof state.execResult === "function"
            ? state.execResult(cell, state)
            : (typeof state.execResult === "string"
              ? state.execResult
              : `CodeNotebookFactory::cellsDML "${cell}" did not return a function or text (found: ${typeof state
                .execResult})`);
          sqlDML.push(codeNotebookCell.insertDML({
            code_notebook_cell_id: nbh.sqlEngineNewUlid,
            notebook_kernel_id: kernelID,
            notebook_name: notebookName,
            cell_name: cell, // the class's method name is the "cell"
            interpretable_code,
            interpretable_code_hash: await gitLikeHash(interpretable_code),
          }, {
            onConflict: nbh
              .SQL`ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
                   interpretable_code = EXCLUDED.interpretable_code,
                   notebook_kernel_id = EXCLUDED.notebook_kernel_id,
                   updated_at = CURRENT_TIMESTAMP,
                   activity_log = ${codeNotebookCell.activityLogDmlPartial()}`,
          }));
        }
      };
      await kernel.run(instance(), irs);
      return sqlDML;
    },
  };
}

export class LargeLanguageModelsPromptsNotebook<
  EmitContext extends SQLa.SqlEmitContext,
> extends CodeNotebook<EmitContext> {
  constructor(
    readonly nbh: SqlNotebookHelpers<EmitContext>,
    readonly bootstrapNB: BootstrapSqlNotebook<EmitContext>,
    readonly constrNB: ConstructionSqlNotebook<EmitContext>,
    readonly queryNB: QuerySqlNotebook<EmitContext>,
  ) {
    super();
  }

  /**
   * Prepares a prompt that will allow the user to "teach" an LLM about this
   * project's "code notebooks" schema and how to interact with it. Once you
   * @returns AI prompt as text that can be used to allow LLMs to generate SQL for you
   */
  "understand notebooks schema"() {
    return () =>
      // deno-fmt-ignore
      codeBlock`
        Understand the following structure of an SQLite database designed to store code notebooks and execution kernels. The database comprises three main tables: 'code_notebook_kernel', 'code_notebook_cell', and 'code_notebook_state'.

        1. 'code_notebook_kernel': This table stores information about various kernels or execution engines. Each record includes a unique kernel ID, kernel name, a description, MIME type, file extension, and other metadata such as creation and update timestamps.

        2. 'code_notebook_cell': This table contains individual notebook cells. Each cell is linked to a kernel in the 'code_notebook_kernel' table via 'notebook_kernel_id'. It includes details like the cell's unique ID, notebook name, cell name, interpretable code, and relevant metadata.

        3. 'code_notebook_state': This table tracks the state transitions of notebook cells. Each record links to a cell in the 'code_notebook_cell' table and includes information about the state transition, such as the previous and new states, transition reason, and timestamps.

        The relationships are as follows: Each cell in 'code_notebook_cell' is associated with a kernel in 'code_notebook_kernel'. The 'code_notebook_state' table tracks changes in the state of each cell, linking back to the 'code_notebook_cell' table.

        Use the following SQLite Schema to generate SQL queries that interact with these tables and once you understand them let me know so I can ask you for help:

        ${this.bootstrapNB.bootstrapDDL().SQL(this.nbh.emitCtx)}
      `;
  }

  "understand service schema"() {
    return () =>
      // deno-fmt-ignore
      codeBlock`
        Understand the following structure of an SQLite database designed to store cybersecurity and compliance data for files in a file system.
        The database is designed to store devices in the 'device' table and entities called 'resources' stored in the immutable append-only
        'uniform_resource' table. Each time files are "walked" they are stored in ingestion session and link back to 'uniform_resource'. Because all
        tables are generally append only and immutable it means that the ingest_session_fs_path_entry table can be used for revision control
        and historical tracking of file changes.

        Use the following SQLite Schema to generate SQL queries that interact with these tables and once you understand them let me know so I can ask you for help:

        ${this.constrNB.v001_once_initialDDL().SQL(this.nbh.emitCtx)}

        ${this.constrNB.v002_fsContentIngestSessionStatsViewDDL().SQL(this.nbh.emitCtx)
      }
      `;
  }
}

export const orchestrableSqlNotebooksNames = [
  "bootstrap",
  "construction",
  "mutation",
  "query",
  "assurance",
] as const;

export type OrchestrableSqlNotebookName =
  typeof orchestrableSqlNotebooksNames[number];

/**
 * Encapsulates instances of all the other notebooks and performs all the work
 * of creating other notebook kernel factories and actually performing
 * operations with those notebooks' cells.
 */
export class SqlNotebooksOrchestrator<EmitContext extends SQLa.SqlEmitContext> {
  readonly bootstrapNBF: ReturnType<
    typeof SQLa.sqlNotebookFactory<
      BootstrapSqlNotebook<EmitContext>,
      EmitContext
    >
  >;
  readonly constructionNBF: ReturnType<
    typeof SQLa.sqlNotebookFactory<
      ConstructionSqlNotebook<EmitContext>,
      EmitContext
    >
  >;
  readonly mutationNBF: ReturnType<
    typeof SQLa.sqlNotebookFactory<
      MutationSqlNotebook<EmitContext>,
      EmitContext
    >
  >;
  readonly queryNBF: ReturnType<
    typeof SQLa.sqlNotebookFactory<QuerySqlNotebook<EmitContext>, EmitContext>
  >;
  readonly assuranceNBF: ReturnType<
    typeof SQLa.sqlNotebookFactory<
      AssuranceSqlNotebook<EmitContext>,
      EmitContext
    >
  >;
  readonly llmPromptsNBF: ReturnType<
    typeof codeNotebookFactory<
      LargeLanguageModelsPromptsNotebook<EmitContext>,
      EmitContext
    >
  >;

  readonly bootstrapNB: BootstrapSqlNotebook<EmitContext>;
  readonly constructionNB: ConstructionSqlNotebook<EmitContext>;
  readonly mutationNB: MutationSqlNotebook<EmitContext>;
  readonly queryNB: QuerySqlNotebook<EmitContext>;
  readonly assuranceNB: AssuranceSqlNotebook<EmitContext>;
  readonly llmPromptsNB: LargeLanguageModelsPromptsNotebook<EmitContext>;

  constructor(readonly nbh: SqlNotebookHelpers<EmitContext>) {
    this.bootstrapNBF = SQLa.sqlNotebookFactory(
      BootstrapSqlNotebook.prototype,
      () => new BootstrapSqlNotebook<EmitContext>(nbh),
    );
    this.constructionNBF = SQLa.sqlNotebookFactory(
      ConstructionSqlNotebook.prototype,
      () => new ConstructionSqlNotebook<EmitContext>(nbh, []),
    );
    this.mutationNBF = SQLa.sqlNotebookFactory(
      MutationSqlNotebook.prototype,
      () => new MutationSqlNotebook<EmitContext>(nbh),
    );
    this.queryNBF = SQLa.sqlNotebookFactory(
      QuerySqlNotebook.prototype,
      () => new QuerySqlNotebook<EmitContext>(nbh),
    );
    this.assuranceNBF = SQLa.sqlNotebookFactory(
      AssuranceSqlNotebook.prototype,
      () => new AssuranceSqlNotebook<EmitContext>(nbh),
    );

    this.bootstrapNB = this.bootstrapNBF.instance();
    this.constructionNB = this.constructionNBF.instance();
    this.mutationNB = this.mutationNBF.instance();
    this.queryNB = this.queryNBF.instance();
    this.assuranceNB = this.assuranceNBF.instance();

    this.llmPromptsNBF = codeNotebookFactory(
      LargeLanguageModelsPromptsNotebook.prototype,
      () =>
        new LargeLanguageModelsPromptsNotebook<EmitContext>(
          nbh,
          this.bootstrapNB,
          this.constructionNB,
          this.queryNB,
        ),
    );
    this.llmPromptsNB = this.llmPromptsNBF.instance();
  }

  separator(cell: string) {
    return {
      executeSqlBehavior: () => ({
        SQL: () => `\n---\n--- Cell: ${cell}\n---\n`,
      }),
    };
  }

  surveilrInfoSchemaDiagram() {
    const { nbh: { modelsGovn, models } } = this;
    const ctx = modelsGovn.sqlEmitContext();
    return typical.diaPUML.plantUmlIE(
      ctx,
      function* () {
        for (const table of models.informationSchema.tables) {
          if (SQLa.isGraphEntityDefinitionSupplier(table)) {
            yield table.graphEntityDefn() as Any; // TODO: why is "Any" required here???
          }
        }
      },
      typical.diaPUML.typicalPlantUmlIeOptions({
        // don't put housekeeping columns in the diagram
        includeEntityAttr: (ea) =>
          [
              "created_at",
              "created_by",
              "updated_at",
              "updated_by",
              "deleted_at",
              "deleted_by",
              "activity_log",
            ].find((c) => c == ea.attr.identity)
            ? false
            : true,
      }),
    ).content;
  }

  notebooksInfoSchemaDiagram() {
    const { nbh: { modelsGovn, models: { codeNbModels } } } = this;
    const ctx = modelsGovn.sqlEmitContext();
    return typical.diaPUML.plantUmlIE(
      ctx,
      function* () {
        for (const table of codeNbModels.informationSchema.tables) {
          if (SQLa.isGraphEntityDefinitionSupplier(table)) {
            yield table.graphEntityDefn() as Any; // TODO: why is "Any" required here???
          }
        }
      },
      typical.diaPUML.typicalPlantUmlIeOptions(),
    ).content;
  }

  async infoSchemaDiagramDML() {
    const { nbh: { models } } = this;
    const { codeNbModels: { codeNotebookCell } } = models;
    const surveilrInfoSchemaDiagram = this.surveilrInfoSchemaDiagram();
    const notebooksInfoSchemaDiagram = this.notebooksInfoSchemaDiagram();
    const options = {
      onConflict: this.nbh
        .SQL`ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
             interpretable_code = EXCLUDED.interpretable_code,
             notebook_kernel_id = EXCLUDED.notebook_kernel_id,
             updated_at = CURRENT_TIMESTAMP,
             activity_log = ${codeNotebookCell.activityLogDmlPartial()}`,
    };
    return [
      codeNotebookCell.insertDML({
        code_notebook_cell_id: this.nbh.sqlEngineNewUlid,
        notebook_kernel_id: "PlantUML",
        notebook_name: SqlNotebooksOrchestrator.prototype.constructor.name,
        cell_name: "surveilrInfoSchemaDiagram",
        interpretable_code: surveilrInfoSchemaDiagram,
        interpretable_code_hash: await gitLikeHash(surveilrInfoSchemaDiagram),
      }, options),
      codeNotebookCell.insertDML({
        code_notebook_cell_id: this.nbh.sqlEngineNewUlid,
        notebook_kernel_id: "PlantUML",
        notebook_name: SqlNotebooksOrchestrator.prototype.constructor.name,
        cell_name: "notebooksInfoSchemaDiagram",
        interpretable_code: notebooksInfoSchemaDiagram,
        interpretable_code_hash: await gitLikeHash(notebooksInfoSchemaDiagram),
      }, options),
    ];
  }

  introspectedCells() {
    const cells: {
      readonly notebook: OrchestrableSqlNotebookName;
      readonly cell: string;
    }[] = [];
    this.bootstrapNBF.kernel.introspectedNB.cells.forEach((cell) => {
      cells.push({ notebook: "bootstrap", cell: cell.nbCellID });
    });
    this.constructionNBF.kernel.introspectedNB.cells.forEach((cell) => {
      cells.push({ notebook: "construction", cell: cell.nbCellID });
    });
    this.mutationNBF.kernel.introspectedNB.cells.forEach((cell) => {
      cells.push({ notebook: "mutation", cell: cell.nbCellID });
    });
    this.queryNBF.kernel.introspectedNB.cells.forEach((cell) => {
      cells.push({ notebook: "query", cell: cell.nbCellID });
    });
    this.assuranceNBF.kernel.introspectedNB.cells.forEach((cell) => {
      cells.push({ notebook: "assurance", cell: cell.nbCellID });
    });
    return cells;
  }

  tblsYAML() {
    const { nbh: { models, models: { codeNbModels } } } = this;
    return [
      {
        identity: "surveilr-state.tbls.auto.yml",
        emit: tbls.tblsConfig(
          function* () {
            for (const table of models.informationSchema.tables) {
              yield table;
            }
          },
          tbls.defaultTblsOptions(),
          { name: "Resource Surveillance State Schema" },
        ),
      },
      {
        identity: "surveilr-code-notebooks.tbls.auto.yml",
        emit: tbls.tblsConfig(
          function* () {
            for (const table of codeNbModels.informationSchema.tables) {
              yield table;
            }
          },
          tbls.defaultTblsOptions(),
          { name: "Resource Surveillance Notebooks Schema" },
        ),
      },
    ];
  }

  async storeNotebookCellsDML() {
    const { codeNbModels: { codeNotebookCell } } = this.nbh.models;
    const ctx = this.nbh.modelsGovn.sqlEmitContext<EmitContext>();
    const sqlDML: SQLa.SqlTextSupplier<EmitContext>[] = [];

    const sqlKernelDML = async <
      Factory extends ReturnType<
        typeof SQLa.sqlNotebookFactory<Any, EmitContext>
      >,
    >(f: Factory, notebookName: string) => {
      // prepare the run state with list of all pages defined and have the kernel
      // traverse the cells and emit (the SQL generator, no SQL is executed)
      const instance = f.instance();
      const irs = await f.kernel.initRunState();
      irs.runState.eventEmitter.afterCell = async (cell, state) => {
        if (state.status == "successful") {
          const interpretable_code =
            SQLa.isSqlTextSupplier<EmitContext>(state.execResult)
              ? state.execResult.SQL(ctx)
              : `storeNotebookCellsDML "${cell}" did not return SQL (found: ${typeof state
                .execResult})`;
          sqlDML.push(codeNotebookCell.insertDML({
            code_notebook_cell_id: this.nbh.sqlEngineNewUlid,
            notebook_kernel_id: "SQL",
            notebook_name: notebookName,
            cell_name: cell, // the class's method name is the "cell"
            interpretable_code,
            interpretable_code_hash: await gitLikeHash(interpretable_code),
          }, {
            onConflict: this.nbh
              .SQL`ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = ${codeNotebookCell.activityLogDmlPartial()}`,
          }));
        }
      };
      await f.kernel.run(instance, irs);
    };

    await sqlKernelDML(
      this.bootstrapNBF as Any,
      BootstrapSqlNotebook.prototype.constructor.name,
    );
    await sqlKernelDML(
      this.constructionNBF as Any,
      ConstructionSqlNotebook.prototype.constructor.name,
    );
    await sqlKernelDML(
      this.mutationNBF as Any,
      MutationSqlNotebook.prototype.constructor.name,
    );
    await sqlKernelDML(
      this.queryNBF as Any,
      QuerySqlNotebook.prototype.constructor.name,
    );
    await sqlKernelDML(
      this.assuranceNBF as Any,
      AssuranceSqlNotebook.prototype.constructor.name,
    );

    sqlDML.push(
      ...(await this.llmPromptsNBF.cellsDML(
        this.nbh,
        "LLM Prompt",
        "LargeLanguageModelsPromptsNotebook",
      )),
    );

    // NOTE: SQLPageNotebook is not stored since its cells are stored in special
    //       `sqlpage_files` table so we don't put them into regular notebooks.

    return this.nbh.SQL`
      ${sqlDML};

      ${await this.infoSchemaDiagramDML()}
      `;
  }
}
