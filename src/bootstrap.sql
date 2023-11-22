CREATE TABLE IF NOT EXISTS "assurance_schema" (
    "assurance_schema_id" TEXT PRIMARY KEY NOT NULL,
    "assurance_type" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "code_json" TEXT CHECK(json_valid(code_json) OR code_json IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "code_notebook_kernel" (
    "code_notebook_kernel_id" TEXT PRIMARY KEY NOT NULL,
    "kernel_name" TEXT NOT NULL,
    "description" TEXT,
    "mime_type" TEXT,
    "file_extn" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("kernel_name")
);
CREATE TABLE IF NOT EXISTS "code_notebook_cell" (
    "code_notebook_cell_id" TEXT PRIMARY KEY NOT NULL,
    "notebook_kernel_id" TEXT NOT NULL,
    "notebook_name" TEXT NOT NULL,
    "cell_name" TEXT NOT NULL,
    "cell_governance" TEXT CHECK(json_valid(cell_governance) OR cell_governance IS NULL),
    "interpretable_code" TEXT NOT NULL,
    "interpretable_code_hash" TEXT NOT NULL,
    "description" TEXT,
    "arguments" TEXT CHECK(json_valid(arguments) OR arguments IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("notebook_kernel_id") REFERENCES "code_notebook_kernel"("code_notebook_kernel_id"),
    UNIQUE("notebook_name", "cell_name", "interpretable_code_hash")
);
CREATE TABLE IF NOT EXISTS "code_notebook_state" (
    "code_notebook_state_id" TEXT PRIMARY KEY NOT NULL,
    "code_notebook_cell_id" TEXT NOT NULL,
    "from_state" TEXT NOT NULL,
    "to_state" TEXT NOT NULL,
    "transition_result" TEXT CHECK(json_valid(transition_result) OR transition_result IS NULL),
    "transition_reason" TEXT,
    "transitioned_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("code_notebook_cell_id") REFERENCES "code_notebook_cell"("code_notebook_cell_id"),
    UNIQUE("code_notebook_cell_id", "from_state", "to_state")
);


;

INSERT INTO "code_notebook_kernel" ("code_notebook_kernel_id", "kernel_name", "description", "mime_type", "file_extn", "elaboration", "governance", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('SQL', 'Dialect-independent ANSI SQL', NULL, 'application/sql', '.sql', NULL, NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(kernel_name) DO UPDATE SET mime_type = EXCLUDED.mime_type, file_extn = EXCLUDED.file_extn;
INSERT INTO "code_notebook_kernel" ("code_notebook_kernel_id", "kernel_name", "description", "mime_type", "file_extn", "elaboration", "governance", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('PlantUML', 'PlantUML ER Diagram', NULL, 'text/vnd.plantuml', '.puml', NULL, NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(kernel_name) DO UPDATE SET mime_type = EXCLUDED.mime_type, file_extn = EXCLUDED.file_extn;
INSERT INTO "code_notebook_kernel" ("code_notebook_kernel_id", "kernel_name", "description", "mime_type", "file_extn", "elaboration", "governance", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('LLM Prompt', 'Large Lanugage Model (LLM) Prompt', NULL, 'text/vnd.netspective.llm-prompt', '.llm-prompt.txt', NULL, NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(kernel_name) DO UPDATE SET mime_type = EXCLUDED.mime_type, file_extn = EXCLUDED.file_extn;

-- store all SQL that is potentially reusable in the database
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'BootstrapSqlNotebook', 'bootstrapDDL', NULL, 'CREATE TABLE IF NOT EXISTS "assurance_schema" (
    "assurance_schema_id" TEXT PRIMARY KEY NOT NULL,
    "assurance_type" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "code_json" TEXT CHECK(json_valid(code_json) OR code_json IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "code_notebook_kernel" (
    "code_notebook_kernel_id" TEXT PRIMARY KEY NOT NULL,
    "kernel_name" TEXT NOT NULL,
    "description" TEXT,
    "mime_type" TEXT,
    "file_extn" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("kernel_name")
);
CREATE TABLE IF NOT EXISTS "code_notebook_cell" (
    "code_notebook_cell_id" TEXT PRIMARY KEY NOT NULL,
    "notebook_kernel_id" TEXT NOT NULL,
    "notebook_name" TEXT NOT NULL,
    "cell_name" TEXT NOT NULL,
    "cell_governance" TEXT CHECK(json_valid(cell_governance) OR cell_governance IS NULL),
    "interpretable_code" TEXT NOT NULL,
    "interpretable_code_hash" TEXT NOT NULL,
    "description" TEXT,
    "arguments" TEXT CHECK(json_valid(arguments) OR arguments IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("notebook_kernel_id") REFERENCES "code_notebook_kernel"("code_notebook_kernel_id"),
    UNIQUE("notebook_name", "cell_name", "interpretable_code_hash")
);
CREATE TABLE IF NOT EXISTS "code_notebook_state" (
    "code_notebook_state_id" TEXT PRIMARY KEY NOT NULL,
    "code_notebook_cell_id" TEXT NOT NULL,
    "from_state" TEXT NOT NULL,
    "to_state" TEXT NOT NULL,
    "transition_result" TEXT CHECK(json_valid(transition_result) OR transition_result IS NULL),
    "transition_reason" TEXT,
    "transitioned_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("code_notebook_cell_id") REFERENCES "code_notebook_cell"("code_notebook_cell_id"),
    UNIQUE("code_notebook_cell_id", "from_state", "to_state")
);


', '462f572072dd9c791b33439efac2ecac8dd5b0b1', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'BootstrapSqlNotebook', 'bootstrapSeedDML', NULL, 'storeNotebookCellsDML "bootstrapSeedDML" did not return SQL (found: object)', 'd5bcdf4a75be14925b1008a47362707c00cb8990', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v001_once_initialDDL', NULL, 'CREATE TABLE IF NOT EXISTS "device" (
    "device_id" ULID PRIMARY KEY NOT NULL,
    "name" TEXT NOT NULL,
    "state" TEXT CHECK(json_valid(state)) NOT NULL,
    "boundary" TEXT NOT NULL,
    "segmentation" TEXT CHECK(json_valid(segmentation) OR segmentation IS NULL),
    "state_sysinfo" TEXT CHECK(json_valid(state_sysinfo) OR state_sysinfo IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("name", "state", "boundary")
);
CREATE TABLE IF NOT EXISTS "behavior" (
    "behavior_id" ULID PRIMARY KEY NOT NULL,
    "device_id" ULID NOT NULL,
    "behavior_name" TEXT NOT NULL,
    "behavior_conf_json" TEXT CHECK(json_valid(behavior_conf_json)) NOT NULL,
    "assurance_schema_id" TEXT,
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("assurance_schema_id") REFERENCES "assurance_schema"("assurance_schema_id"),
    UNIQUE("device_id", "behavior_name")
);
CREATE TABLE IF NOT EXISTS "ur_walk_session" (
    "ur_walk_session_id" ULID PRIMARY KEY NOT NULL,
    "device_id" ULID NOT NULL,
    "behavior_id" ULID,
    "behavior_json" TEXT CHECK(json_valid(behavior_json) OR behavior_json IS NULL),
    "walk_started_at" TIMESTAMP NOT NULL,
    "walk_finished_at" TIMESTAMP,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("behavior_id") REFERENCES "behavior"("behavior_id"),
    UNIQUE("device_id", "created_at")
);
CREATE TABLE IF NOT EXISTS "ur_walk_session_path" (
    "ur_walk_session_path_id" ULID PRIMARY KEY NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "root_path" TEXT NOT NULL,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("walk_session_id") REFERENCES "ur_walk_session"("ur_walk_session_id"),
    UNIQUE("walk_session_id", "root_path", "created_at")
);
CREATE TABLE IF NOT EXISTS "uniform_resource" (
    "uniform_resource_id" ULID PRIMARY KEY NOT NULL,
    "device_id" ULID NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "walk_path_id" ULID NOT NULL,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "last_modified_at" INTEGER,
    "content_fm_body_attrs" TEXT CHECK(json_valid(content_fm_body_attrs) OR content_fm_body_attrs IS NULL),
    "frontmatter" TEXT CHECK(json_valid(frontmatter) OR frontmatter IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("walk_session_id") REFERENCES "ur_walk_session"("ur_walk_session_id"),
    FOREIGN KEY("walk_path_id") REFERENCES "ur_walk_session_path"("ur_walk_session_path_id"),
    UNIQUE("device_id", "content_digest", "uri", "size_bytes", "last_modified_at")
);
CREATE TABLE IF NOT EXISTS "uniform_resource_transform" (
    "uniform_resource_transform_id" ULID PRIMARY KEY NOT NULL,
    "uniform_resource_id" ULID NOT NULL,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id"),
    UNIQUE("uniform_resource_id", "content_digest", "nature", "size_bytes")
);
CREATE TABLE IF NOT EXISTS "ur_walk_session_path_fs_entry" (
    "ur_walk_session_path_fs_entry_id" ULID PRIMARY KEY NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "walk_path_id" ULID NOT NULL,
    "uniform_resource_id" ULID,
    "file_path_abs" TEXT NOT NULL,
    "file_path_rel_parent" TEXT NOT NULL,
    "file_path_rel" TEXT NOT NULL,
    "file_basename" TEXT NOT NULL,
    "file_extn" TEXT,
    "captured_executable" TEXT CHECK(json_valid(captured_executable) OR captured_executable IS NULL),
    "ur_status" TEXT,
    "ur_diagnostics" TEXT CHECK(json_valid(ur_diagnostics) OR ur_diagnostics IS NULL),
    "ur_transformations" TEXT CHECK(json_valid(ur_transformations) OR ur_transformations IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("walk_session_id") REFERENCES "ur_walk_session"("ur_walk_session_id"),
    FOREIGN KEY("walk_path_id") REFERENCES "ur_walk_session_path"("ur_walk_session_path_id"),
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id")
);

CREATE INDEX IF NOT EXISTS "idx_device__name__state" ON "device"("name", "state");
CREATE INDEX IF NOT EXISTS "idx_ur_walk_session_path__walk_session_id__root_path" ON "ur_walk_session_path"("walk_session_id", "root_path");
CREATE INDEX IF NOT EXISTS "idx_uniform_resource__device_id__uri" ON "uniform_resource"("device_id", "uri");
CREATE INDEX IF NOT EXISTS "idx_uniform_resource_transform__uniform_resource_id__content_digest" ON "uniform_resource_transform"("uniform_resource_id", "content_digest");
CREATE INDEX IF NOT EXISTS "idx_ur_walk_session_path_fs_entry__walk_session_id__file_path_abs" ON "ur_walk_session_path_fs_entry"("walk_session_id", "file_path_abs");
', '03969c5bd12130ea7544ece1bd3c4bc91944db4b', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v002_fsContentWalkSessionStatsViewDDL', NULL, 'DROP VIEW IF EXISTS "fs_content_walk_session_stats";
CREATE VIEW IF NOT EXISTS "fs_content_walk_session_stats" AS
    WITH Summary AS (
        SELECT
            device.device_id AS device_id,
            ur_walk_session.ur_walk_session_id AS walk_session_id,
            ur_walk_session.walk_started_at AS walk_session_started_at,
            ur_walk_session.walk_finished_at AS walk_session_finished_at,
            COALESCE(ur_walk_session_path_fs_entry.file_extn, '''') AS file_extension,
            ur_walk_session_path.ur_walk_session_path_id as walk_session_path_id,
            ur_walk_session_path.root_path AS walk_session_root_path,
            COUNT(ur_walk_session_path_fs_entry.uniform_resource_id) AS total_file_count,
            SUM(CASE WHEN uniform_resource.content IS NOT NULL THEN 1 ELSE 0 END) AS file_count_with_content,
            SUM(CASE WHEN uniform_resource.frontmatter IS NOT NULL THEN 1 ELSE 0 END) AS file_count_with_frontmatter,
            MIN(uniform_resource.size_bytes) AS min_file_size_bytes,
            AVG(uniform_resource.size_bytes) AS average_file_size_bytes,
            MAX(uniform_resource.size_bytes) AS max_file_size_bytes,
            MIN(uniform_resource.last_modified_at) AS oldest_file_last_modified_datetime,
            MAX(uniform_resource.last_modified_at) AS youngest_file_last_modified_datetime
        FROM
            ur_walk_session
        JOIN
            device ON ur_walk_session.device_id = device.device_id
        LEFT JOIN
            ur_walk_session_path ON ur_walk_session.ur_walk_session_id = ur_walk_session_path.walk_session_id
        LEFT JOIN
            ur_walk_session_path_fs_entry ON ur_walk_session_path.ur_walk_session_path_id = ur_walk_session_path_fs_entry.walk_path_id
        LEFT JOIN
            uniform_resource ON ur_walk_session_path_fs_entry.uniform_resource_id = uniform_resource.uniform_resource_id
        GROUP BY
            device.device_id,
            ur_walk_session.ur_walk_session_id,
            ur_walk_session.walk_started_at,
            ur_walk_session.walk_finished_at,
            ur_walk_session_path_fs_entry.file_extn,
            ur_walk_session_path.root_path
    )
    SELECT
        device_id,
        walk_session_id,
        walk_session_started_at,
        walk_session_finished_at,
        file_extension,
        walk_session_path_id,
        walk_session_root_path,
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
        walk_session_finished_at,
        file_extension;
    ', '2a13ba110d9f0dda21cf7442e7c9c8cc1fb2b58b', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v002_fsContentWalkSessionStatsLatestViewDDL', NULL, 'DROP VIEW IF EXISTS "fs_content_walk_session_stats_latest";
CREATE VIEW IF NOT EXISTS "fs_content_walk_session_stats_latest" AS
    SELECT fs.*
      FROM fs_content_walk_session_stats AS fs
      JOIN (  SELECT ur_walk_session.ur_walk_session_id AS latest_session_id
                FROM ur_walk_session
            ORDER BY ur_walk_session.walk_finished_at DESC
               LIMIT 1) AS latest
        ON fs.walk_session_id = latest.latest_session_id;', 'f2b7a41a567cab6a59d6dc57a223ec053d0f2276', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v002_urWalkSessionIssueViewDDL', NULL, 'DROP VIEW IF EXISTS "ur_walk_session_issue";
CREATE VIEW IF NOT EXISTS "ur_walk_session_issue" AS
      SELECT us.device_id,
             us.ur_walk_session_id,
             usp.ur_walk_session_path_id,
             usp.root_path,
             ufs.ur_walk_session_path_fs_entry_id,
             ufs.file_path_abs,
             ufs.ur_status,
             ufs.ur_diagnostics
        FROM ur_walk_session_path_fs_entry ufs
        JOIN ur_walk_session_path usp ON ufs.walk_path_id = usp.ur_walk_session_path_id
        JOIN ur_walk_session us ON usp.walk_session_id = us.ur_walk_session_id
       WHERE ufs.ur_status IS NOT NULL
    GROUP BY us.device_id, 
             us.ur_walk_session_id, 
             usp.ur_walk_session_path_id, 
             usp.root_path, 
             ufs.ur_walk_session_path_fs_entry_id, 
             ufs.file_path_abs, 
             ufs.ur_status, 
             ufs.ur_diagnostics;', '5b248ab0b7f6fafa0ad64a05d4327596e9df0039', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'QuerySqlNotebook', 'infoSchema', NULL, 'SELECT tbl_name AS table_name,
       c.cid AS column_id,
       c.name AS column_name,
       c."type" AS "type",
       c."notnull" AS "notnull",
       c.dflt_value as "default_value",
       c.pk AS primary_key
  FROM sqlite_master m,
       pragma_table_info(m.tbl_name) c
 WHERE m.type = ''table'';', 'b51a0cbc609deb9b4046bb02b670e7686ac678fe', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'QuerySqlNotebook', 'infoSchemaOsQueryATCs', NULL, 'WITH table_columns AS (
    SELECT m.tbl_name AS table_name,
           group_concat(c.name) AS column_names_for_select,
           json_group_array(c.name) AS column_names_for_atc_json
      FROM sqlite_master m,
           pragma_table_info(m.tbl_name) c
     WHERE m.type = ''table''
  GROUP BY m.tbl_name
),
target AS (
  -- set SQLite parameter :osquery_atc_path to assign a different path
  SELECT COALESCE(:osquery_atc_path, ''SQLITEDB_PATH'') AS path
),
table_query AS (
    SELECT table_name,
           ''SELECT '' || column_names_for_select || '' FROM '' || table_name AS query,
           column_names_for_atc_json
      FROM table_columns
)
SELECT json_object(''auto_table_construction'',
          json_group_object(
              table_name,
              json_object(
                  ''query'', query,
                  ''columns'', json(column_names_for_atc_json),
                  ''path'', path
              )
          )
       ) AS osquery_auto_table_construction
  FROM table_query, target;', '375b0b4fe291ebac350dc4ed9304f17b57ba8e44', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'QuerySqlNotebook', 'infoSchemaMarkdown', NULL, '-- TODO: https://github.com/lovasoa/SQLpage/discussions/109#discussioncomment-7359513
--       see the above for how to fix for SQLPage but figure out to use the same SQL
--       in and out of SQLPage (maybe do what Ophir said in discussion and create
--       custom output for SQLPage using componetns?)
WITH TableInfo AS (
  SELECT
    m.tbl_name AS table_name,
    CASE WHEN c.pk THEN ''*'' ELSE '''' END AS is_primary_key,
    c.name AS column_name,
    c."type" AS column_type,
    CASE WHEN c."notnull" THEN ''*'' ELSE '''' END AS not_null,
    COALESCE(c.dflt_value, '''') AS default_value,
    COALESCE((SELECT pfkl."table" || ''.'' || pfkl."to" FROM pragma_foreign_key_list(m.tbl_name) AS pfkl WHERE pfkl."from" = c.name), '''') as fk_refs,
    ROW_NUMBER() OVER (PARTITION BY m.tbl_name ORDER BY c.cid) AS row_num
  FROM sqlite_master m JOIN pragma_table_info(m.tbl_name) c ON 1=1
  WHERE m.type = ''table''
  ORDER BY table_name, row_num
),
Views AS (
  SELECT ''## Views '' AS markdown_output
  UNION ALL
  SELECT ''| View | Column | Type |'' AS markdown_output
  UNION ALL
  SELECT ''| ---- | ------ |----- |'' AS markdown_output
  UNION ALL
  SELECT ''| '' || tbl_name || '' | '' || c.name || '' | '' || c."type" || '' | ''
  FROM
    sqlite_master m,
    pragma_table_info(m.tbl_name) c
  WHERE
    m.type = ''view''
),
Indexes AS (
  SELECT ''## Indexes'' AS markdown_output
  UNION ALL
  SELECT ''| Table | Index | Columns |'' AS markdown_output
  UNION ALL
  SELECT ''| ----- | ----- | ------- |'' AS markdown_output
  UNION ALL
  SELECT ''| '' ||  m.name || '' | '' || il.name || '' | '' || group_concat(ii.name, '', '') || '' |'' AS markdown_output
  FROM sqlite_master as m,
    pragma_index_list(m.name) AS il,
    pragma_index_info(il.name) AS ii
  WHERE
    m.type = ''table''
  GROUP BY
    m.name,
    il.name
)
SELECT
    markdown_output AS info_schema_markdown
FROM
  (
    SELECT ''## Tables'' AS markdown_output
    UNION ALL
    SELECT
      CASE WHEN ti.row_num = 1 THEN ''
### `'' || ti.table_name || ''` Table
| PK | Column | Type | Req? | Default | References |
| -- | ------ | ---- | ---- | ------- | ---------- |
'' ||
        ''| '' || is_primary_key || '' | '' || ti.column_name || '' | '' || ti.column_type || '' | '' || ti.not_null || '' | '' || ti.default_value || '' | '' || ti.fk_refs || '' |''
      ELSE
        ''| '' || is_primary_key || '' | '' || ti.column_name || '' | '' || ti.column_type || '' | '' || ti.not_null || '' | '' || ti.default_value || '' | '' || ti.fk_refs || '' |''
      END
    FROM TableInfo ti
    UNION ALL SELECT ''''
    UNION ALL SELECT * FROM	Views
    UNION ALL SELECT ''''
    UNION ALL SELECT * FROM Indexes
);', 'cbe800ad94b1b90f0045a78a03e9da246e46ff75', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'QuerySqlNotebook', 'htmlAnchors', NULL, '-- loadExtnSQL not provided to load ''asg017/html/html0''

-- find all HTML files in the uniform_resource table and return
-- each file and the anchors'' labels and hrefs in that file
-- TODO: create a table called fs_content_html_anchor to store this data after inserting it into uniform_resource
--       so that simple HTML lookups do not require the html0 extension to be loaded
WITH html_content AS (
  SELECT uniform_resource_id, content, content_digest, file_path, file_extn FROM uniform_resource WHERE nature = ''html''
),
html AS (
  SELECT file_path,
         text as label,
         html_attribute_get(html, ''a'', ''href'') as href
    FROM html_content, html_each(html_content.content, ''a'')
)
SELECT * FROM html;
      ', '7d9cdf2286d82fae97cb3652e6b17744ebf7c817', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'QuerySqlNotebook', 'htmlHeadMeta', NULL, '-- loadExtnSQL not provided to load ''asg017/html/html0''

-- find all HTML files in the uniform_resource table and return
-- each file and the <head><meta name="key" content="value"> pair
-- TODO: create a table called resource_html_head_meta to store this data after inserting it into uniform_resource
--       so that simple HTML lookups do not require the html0 extension to be loaded
WITH html_content AS (
  SELECT uniform_resource_id, content, content_digest, file_path, file_extn FROM uniform_resource WHERE nature = ''html''
),
html AS (
  SELECT file_path,
         html_attribute_get(html, ''meta'', ''name'') as key,
         html_attribute_get(html, ''meta'', ''content'') as value,
         html
    FROM html_content, html_each(html_content.content, ''head meta'')
   WHERE key IS NOT NULL
)
SELECT * FROM html;
      ', '70d990a17e2ce8d0ee2cf6dd368390c8b1db7645', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'AssuranceSqlNotebook', 'test1', NULL, 'WITH test_plan AS (
    SELECT ''1..1'' AS tap_output
),
test1 AS (  -- Check if the ''fileio'' extension is loaded by calling the ''readfile'' function
    SELECT
        CASE
            WHEN readfile(''README.md'') IS NOT NULL THEN ''ok 1 - fileio extension is loaded.''
            ELSE ''not ok 1 - fileio extension is not loaded.''
        END AS tap_output
    FROM (SELECT 1) -- This is a dummy table of one row to ensure the SELECT runs.
)
SELECT tap_output FROM test_plan
UNION ALL
SELECT tap_output FROM test1;', '083111d6c0f2b4b0670eee784f113ddd8619187d', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'LLM Prompt', 'LargeLanguageModelsPromptsNotebook', 'understand notebooks schema', NULL, 'Understand the following structure of an SQLite database designed to store code notebooks and execution kernels. The database comprises three main tables: ''code_notebook_kernel'', ''code_notebook_cell'', and ''code_notebook_state''.

1. ''code_notebook_kernel'': This table stores information about various kernels or execution engines. Each record includes a unique kernel ID, kernel name, a description, MIME type, file extension, and other metadata such as creation and update timestamps.

2. ''code_notebook_cell'': This table contains individual notebook cells. Each cell is linked to a kernel in the ''code_notebook_kernel'' table via ''notebook_kernel_id''. It includes details like the cell''s unique ID, notebook name, cell name, interpretable code, and relevant metadata.

3. ''code_notebook_state'': This table tracks the state transitions of notebook cells. Each record links to a cell in the ''code_notebook_cell'' table and includes information about the state transition, such as the previous and new states, transition reason, and timestamps.

The relationships are as follows: Each cell in ''code_notebook_cell'' is associated with a kernel in ''code_notebook_kernel''. The ''code_notebook_state'' table tracks changes in the state of each cell, linking back to the ''code_notebook_cell'' table. 

Use the following SQLite Schema to generate SQL queries that interact with these tables and once you understand them let me know so I can ask you for help:

CREATE TABLE IF NOT EXISTS "assurance_schema" (
    "assurance_schema_id" TEXT PRIMARY KEY NOT NULL,
    "assurance_type" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "code_json" TEXT CHECK(json_valid(code_json) OR code_json IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "code_notebook_kernel" (
    "code_notebook_kernel_id" TEXT PRIMARY KEY NOT NULL,
    "kernel_name" TEXT NOT NULL,
    "description" TEXT,
    "mime_type" TEXT,
    "file_extn" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("kernel_name")
);
CREATE TABLE IF NOT EXISTS "code_notebook_cell" (
    "code_notebook_cell_id" TEXT PRIMARY KEY NOT NULL,
    "notebook_kernel_id" TEXT NOT NULL,
    "notebook_name" TEXT NOT NULL,
    "cell_name" TEXT NOT NULL,
    "cell_governance" TEXT CHECK(json_valid(cell_governance) OR cell_governance IS NULL),
    "interpretable_code" TEXT NOT NULL,
    "interpretable_code_hash" TEXT NOT NULL,
    "description" TEXT,
    "arguments" TEXT CHECK(json_valid(arguments) OR arguments IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("notebook_kernel_id") REFERENCES "code_notebook_kernel"("code_notebook_kernel_id"),
    UNIQUE("notebook_name", "cell_name", "interpretable_code_hash")
);
CREATE TABLE IF NOT EXISTS "code_notebook_state" (
    "code_notebook_state_id" TEXT PRIMARY KEY NOT NULL,
    "code_notebook_cell_id" TEXT NOT NULL,
    "from_state" TEXT NOT NULL,
    "to_state" TEXT NOT NULL,
    "transition_result" TEXT CHECK(json_valid(transition_result) OR transition_result IS NULL),
    "transition_reason" TEXT,
    "transitioned_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("code_notebook_cell_id") REFERENCES "code_notebook_cell"("code_notebook_cell_id"),
    UNIQUE("code_notebook_cell_id", "from_state", "to_state")
);



      ', '8c96a6667bc2dc4ca8df2b210292ed07609703f9', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
                   interpretable_code = EXCLUDED.interpretable_code,
                   notebook_kernel_id = EXCLUDED.notebook_kernel_id,
                   updated_at = CURRENT_TIMESTAMP,
                   activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'LLM Prompt', 'LargeLanguageModelsPromptsNotebook', 'understand service schema', NULL, 'Understand the following structure of an SQLite database designed to store cybersecurity and compliance data for files in a file system.
The database is designed to store devices in the ''device'' table and entities called ''resources'' stored in the immutable append-only 
''uniform_resource'' table. Each time files are "walked" they are stored in sessions and link back to ''uniform_resource''. Because all
tables are generally append only and immutable it means that the walk_session_path_fs_entry table can be used for revision control
and historical tracking of file changes.

Use the following SQLite Schema to generate SQL queries that interact with these tables and once you understand them let me know so I can ask you for help:

CREATE TABLE IF NOT EXISTS "device" (
    "device_id" ULID PRIMARY KEY NOT NULL,
    "name" TEXT NOT NULL,
    "state" TEXT CHECK(json_valid(state)) NOT NULL,
    "boundary" TEXT NOT NULL,
    "segmentation" TEXT CHECK(json_valid(segmentation) OR segmentation IS NULL),
    "state_sysinfo" TEXT CHECK(json_valid(state_sysinfo) OR state_sysinfo IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("name", "state", "boundary")
);
CREATE TABLE IF NOT EXISTS "behavior" (
    "behavior_id" ULID PRIMARY KEY NOT NULL,
    "device_id" ULID NOT NULL,
    "behavior_name" TEXT NOT NULL,
    "behavior_conf_json" TEXT CHECK(json_valid(behavior_conf_json)) NOT NULL,
    "assurance_schema_id" TEXT,
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("assurance_schema_id") REFERENCES "assurance_schema"("assurance_schema_id"),
    UNIQUE("device_id", "behavior_name")
);
CREATE TABLE IF NOT EXISTS "ur_walk_session" (
    "ur_walk_session_id" ULID PRIMARY KEY NOT NULL,
    "device_id" ULID NOT NULL,
    "behavior_id" ULID,
    "behavior_json" TEXT CHECK(json_valid(behavior_json) OR behavior_json IS NULL),
    "walk_started_at" TIMESTAMP NOT NULL,
    "walk_finished_at" TIMESTAMP,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("behavior_id") REFERENCES "behavior"("behavior_id"),
    UNIQUE("device_id", "created_at")
);
CREATE TABLE IF NOT EXISTS "ur_walk_session_path" (
    "ur_walk_session_path_id" ULID PRIMARY KEY NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "root_path" TEXT NOT NULL,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("walk_session_id") REFERENCES "ur_walk_session"("ur_walk_session_id"),
    UNIQUE("walk_session_id", "root_path", "created_at")
);
CREATE TABLE IF NOT EXISTS "uniform_resource" (
    "uniform_resource_id" ULID PRIMARY KEY NOT NULL,
    "device_id" ULID NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "walk_path_id" ULID NOT NULL,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "last_modified_at" INTEGER,
    "content_fm_body_attrs" TEXT CHECK(json_valid(content_fm_body_attrs) OR content_fm_body_attrs IS NULL),
    "frontmatter" TEXT CHECK(json_valid(frontmatter) OR frontmatter IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("walk_session_id") REFERENCES "ur_walk_session"("ur_walk_session_id"),
    FOREIGN KEY("walk_path_id") REFERENCES "ur_walk_session_path"("ur_walk_session_path_id"),
    UNIQUE("device_id", "content_digest", "uri", "size_bytes", "last_modified_at")
);
CREATE TABLE IF NOT EXISTS "uniform_resource_transform" (
    "uniform_resource_transform_id" ULID PRIMARY KEY NOT NULL,
    "uniform_resource_id" ULID NOT NULL,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id"),
    UNIQUE("uniform_resource_id", "content_digest", "nature", "size_bytes")
);
CREATE TABLE IF NOT EXISTS "ur_walk_session_path_fs_entry" (
    "ur_walk_session_path_fs_entry_id" ULID PRIMARY KEY NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "walk_path_id" ULID NOT NULL,
    "uniform_resource_id" ULID,
    "file_path_abs" TEXT NOT NULL,
    "file_path_rel_parent" TEXT NOT NULL,
    "file_path_rel" TEXT NOT NULL,
    "file_basename" TEXT NOT NULL,
    "file_extn" TEXT,
    "captured_executable" TEXT CHECK(json_valid(captured_executable) OR captured_executable IS NULL),
    "ur_status" TEXT,
    "ur_diagnostics" TEXT CHECK(json_valid(ur_diagnostics) OR ur_diagnostics IS NULL),
    "ur_transformations" TEXT CHECK(json_valid(ur_transformations) OR ur_transformations IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("walk_session_id") REFERENCES "ur_walk_session"("ur_walk_session_id"),
    FOREIGN KEY("walk_path_id") REFERENCES "ur_walk_session_path"("ur_walk_session_path_id"),
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id")
);

CREATE INDEX IF NOT EXISTS "idx_device__name__state" ON "device"("name", "state");
CREATE INDEX IF NOT EXISTS "idx_ur_walk_session_path__walk_session_id__root_path" ON "ur_walk_session_path"("walk_session_id", "root_path");
CREATE INDEX IF NOT EXISTS "idx_uniform_resource__device_id__uri" ON "uniform_resource"("device_id", "uri");
CREATE INDEX IF NOT EXISTS "idx_uniform_resource_transform__uniform_resource_id__content_digest" ON "uniform_resource_transform"("uniform_resource_id", "content_digest");
CREATE INDEX IF NOT EXISTS "idx_ur_walk_session_path_fs_entry__walk_session_id__file_path_abs" ON "ur_walk_session_path_fs_entry"("walk_session_id", "file_path_abs");


DROP VIEW IF EXISTS "fs_content_walk_session_stats";
CREATE VIEW IF NOT EXISTS "fs_content_walk_session_stats" AS
    WITH Summary AS (
        SELECT
            device.device_id AS device_id,
            ur_walk_session.ur_walk_session_id AS walk_session_id,
            ur_walk_session.walk_started_at AS walk_session_started_at,
            ur_walk_session.walk_finished_at AS walk_session_finished_at,
            COALESCE(ur_walk_session_path_fs_entry.file_extn, '''') AS file_extension,
            ur_walk_session_path.ur_walk_session_path_id as walk_session_path_id,
            ur_walk_session_path.root_path AS walk_session_root_path,
            COUNT(ur_walk_session_path_fs_entry.uniform_resource_id) AS total_file_count,
            SUM(CASE WHEN uniform_resource.content IS NOT NULL THEN 1 ELSE 0 END) AS file_count_with_content,
            SUM(CASE WHEN uniform_resource.frontmatter IS NOT NULL THEN 1 ELSE 0 END) AS file_count_with_frontmatter,
            MIN(uniform_resource.size_bytes) AS min_file_size_bytes,
            AVG(uniform_resource.size_bytes) AS average_file_size_bytes,
            MAX(uniform_resource.size_bytes) AS max_file_size_bytes,
            MIN(uniform_resource.last_modified_at) AS oldest_file_last_modified_datetime,
            MAX(uniform_resource.last_modified_at) AS youngest_file_last_modified_datetime
        FROM
            ur_walk_session
        JOIN
            device ON ur_walk_session.device_id = device.device_id
        LEFT JOIN
            ur_walk_session_path ON ur_walk_session.ur_walk_session_id = ur_walk_session_path.walk_session_id
        LEFT JOIN
            ur_walk_session_path_fs_entry ON ur_walk_session_path.ur_walk_session_path_id = ur_walk_session_path_fs_entry.walk_path_id
        LEFT JOIN
            uniform_resource ON ur_walk_session_path_fs_entry.uniform_resource_id = uniform_resource.uniform_resource_id
        GROUP BY
            device.device_id,
            ur_walk_session.ur_walk_session_id,
            ur_walk_session.walk_started_at,
            ur_walk_session.walk_finished_at,
            ur_walk_session_path_fs_entry.file_extn,
            ur_walk_session_path.root_path
    )
    SELECT
        device_id,
        walk_session_id,
        walk_session_started_at,
        walk_session_finished_at,
        file_extension,
        walk_session_path_id,
        walk_session_root_path,
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
        walk_session_finished_at,
        file_extension;
    
      ', '4b0e6dcb614d8683c47304a8370d64eee463f968', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
                   interpretable_code = EXCLUDED.interpretable_code,
                   notebook_kernel_id = EXCLUDED.notebook_kernel_id,
                   updated_at = CURRENT_TIMESTAMP,
                   activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));;

INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'PlantUML', 'SqlNotebooksOrchestrator', 'surveilrInfoSchemaDiagram', NULL, '@startuml IE
  hide circle
  skinparam linetype ortho
  skinparam roundcorner 20
  skinparam class {
    BackgroundColor White
    ArrowColor Silver
    BorderColor Silver
    FontColor Black
    FontSize 12
  }

  entity "device" as device {
    * **device_id**: ULID
    --
    * name: TEXT
    * state: TEXT
    * boundary: TEXT
      segmentation: TEXT
      state_sysinfo: TEXT
      elaboration: TEXT
  }

  entity "behavior" as behavior {
    * **behavior_id**: ULID
    --
    * device_id: ULID
    * behavior_name: TEXT
    * behavior_conf_json: TEXT
      assurance_schema_id: TEXT
      governance: TEXT
  }

  entity "ur_walk_session" as ur_walk_session {
    * **ur_walk_session_id**: ULID
    --
    * device_id: ULID
      behavior_id: ULID
      behavior_json: TEXT
    * walk_started_at: TIMESTAMP
      walk_finished_at: TIMESTAMP
      elaboration: TEXT
  }

  entity "ur_walk_session_path" as ur_walk_session_path {
    * **ur_walk_session_path_id**: ULID
    --
    * walk_session_id: ULID
    * root_path: TEXT
      elaboration: TEXT
  }

  entity "uniform_resource" as uniform_resource {
    * **uniform_resource_id**: ULID
    --
    * device_id: ULID
    * walk_session_id: ULID
    * walk_path_id: ULID
    * uri: TEXT
    * content_digest: TEXT
      content: BLOB
      nature: TEXT
      size_bytes: INTEGER
      last_modified_at: INTEGER
      content_fm_body_attrs: TEXT
      frontmatter: TEXT
      elaboration: TEXT
  }

  entity "uniform_resource_transform" as uniform_resource_transform {
    * **uniform_resource_transform_id**: ULID
    --
    * uniform_resource_id: ULID
    * uri: TEXT
    * content_digest: TEXT
      content: BLOB
      nature: TEXT
      size_bytes: INTEGER
      elaboration: TEXT
  }

  entity "ur_walk_session_path_fs_entry" as ur_walk_session_path_fs_entry {
    * **ur_walk_session_path_fs_entry_id**: ULID
    --
    * walk_session_id: ULID
    * walk_path_id: ULID
      uniform_resource_id: ULID
    * file_path_abs: TEXT
    * file_path_rel_parent: TEXT
    * file_path_rel: TEXT
    * file_basename: TEXT
      file_extn: TEXT
      captured_executable: TEXT
      ur_status: TEXT
      ur_diagnostics: TEXT
      ur_transformations: TEXT
      elaboration: TEXT
  }

  device |o..o{ behavior
  device |o..o{ ur_walk_session
  behavior |o..o{ ur_walk_session
  ur_walk_session |o..o{ ur_walk_session_path
  device |o..o{ uniform_resource
  ur_walk_session |o..o{ uniform_resource
  ur_walk_session_path |o..o{ uniform_resource
  uniform_resource |o..o{ uniform_resource_transform
  ur_walk_session |o..o{ ur_walk_session_path_fs_entry
  ur_walk_session_path |o..o{ ur_walk_session_path_fs_entry
  uniform_resource |o..o{ ur_walk_session_path_fs_entry
@enduml', '6acb64bf8aa60fcc26aab41dafd2fc008218b2d8', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
             interpretable_code = EXCLUDED.interpretable_code,
             notebook_kernel_id = EXCLUDED.notebook_kernel_id,
             updated_at = CURRENT_TIMESTAMP,
             activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'PlantUML', 'SqlNotebooksOrchestrator', 'notebooksInfoSchemaDiagram', NULL, '@startuml IE
  hide circle
  skinparam linetype ortho
  skinparam roundcorner 20
  skinparam class {
    BackgroundColor White
    ArrowColor Silver
    BorderColor Silver
    FontColor Black
    FontSize 12
  }

  entity "assurance_schema" as assurance_schema {
    * **assurance_schema_id**: TEXT
    --
    * assurance_type: TEXT
    * code: TEXT
      code_json: TEXT
      governance: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  entity "code_notebook_kernel" as code_notebook_kernel {
    * **code_notebook_kernel_id**: TEXT
    --
    * kernel_name: TEXT
      description: TEXT
      mime_type: TEXT
      file_extn: TEXT
      elaboration: TEXT
      governance: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  entity "code_notebook_cell" as code_notebook_cell {
    * **code_notebook_cell_id**: TEXT
    --
    * notebook_kernel_id: TEXT
    * notebook_name: TEXT
    * cell_name: TEXT
      cell_governance: TEXT
    * interpretable_code: TEXT
    * interpretable_code_hash: TEXT
      description: TEXT
      arguments: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  entity "code_notebook_state" as code_notebook_state {
    * **code_notebook_state_id**: TEXT
    --
    * code_notebook_cell_id: TEXT
    * from_state: TEXT
    * to_state: TEXT
      transition_result: TEXT
      transition_reason: TEXT
      transitioned_at: TIMESTAMP
      elaboration: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  code_notebook_kernel |o..o{ code_notebook_cell
  code_notebook_cell |o..o{ code_notebook_state
@enduml', 'de749b44284019413053f78449e7f6ed54a3bb25', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
             interpretable_code = EXCLUDED.interpretable_code,
             notebook_kernel_id = EXCLUDED.notebook_kernel_id,
             updated_at = CURRENT_TIMESTAMP,
             activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
;

-- insert SQLPage content for diagnostics and web server
CREATE TABLE IF NOT EXISTS "sqlpage_files" (
    "path" TEXT PRIMARY KEY NOT NULL,
    "contents" TEXT NOT NULL,
    "last_modified" TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('index.sql', 'SELECT
  ''list'' as component,
  ''Get started: where to go from here ?'' as title,
  ''Here are some useful links to get you started with SQLPage.'' as description;
SELECT ''Content Walk Session Statistics'' as title,
  ''fsc-walk-session-stats.sql'' as link,
  ''TODO'' as description,
  ''green'' as color,
  ''download'' as icon;
SELECT ''MIME Types'' as title,
  ''mime-types.sql'' as link,
  ''TODO'' as description,
  ''blue'' as color,
  ''download'' as icon;
SELECT ''Stored SQL Notebooks'' as title,
  ''notebooks.sql'' as link,
  ''TODO'' as description,
  ''blue'' as color,
  ''download'' as icon;
SELECT ''Information Schema'' as title,
  ''info-schema.sql'' as link,
  ''TODO'' as description,
  ''blue'' as color,
  ''download'' as icon;', (CURRENT_TIMESTAMP)) ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP;
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('fsc-walk-session-stats.sql', 'SELECT ''table'' as component, 1 as search, 1 as sort;
SELECT walk_datetime, file_extn, total_count, with_content, with_frontmatter, average_size from fs_content_walk_session_stats;', (CURRENT_TIMESTAMP)) ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP;
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('mime-types.sql', 'SELECT ''table'' as component, 1 as search, 1 as sort;
SELECT name, file_extn, description from mime_type;', (CURRENT_TIMESTAMP)) ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP;
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('notebooks.sql', 'SELECT ''table'' as component, ''Cell'' as markdown, 1 as search, 1 as sort;
SELECT "notebook_name",
       ''['' || "cell_name" || ''](notebook-cell.sql?notebook='' ||  "notebook_name" || ''&cell='' || "cell_name" || '')'' as Cell
  FROM code_notebook_cell;', (CURRENT_TIMESTAMP)) ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP;
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('notebook-cell.sql', 'SELECT ''text'' as component,
       $notebook || ''.'' || $cell as title,
       ''```sql
'' || "interpretable_code" || ''
```'' as contents_md
 FROM code_notebook_cell
WHERE "notebook_name" = $notebook
  AND "cell_name" = $cell;', (CURRENT_TIMESTAMP)) ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP;
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('info-schema.sql', '-- TODO: https://github.com/lovasoa/SQLpage/discussions/109#discussioncomment-7359513
--       see the above for how to fix for SQLPage but figure out to use the same SQL
--       in and out of SQLPage (maybe do what Ophir said in discussion and create
--       custom output for SQLPage using componetns?)
WITH TableInfo AS (
  SELECT
    m.tbl_name AS table_name,
    CASE WHEN c.pk THEN ''*'' ELSE '''' END AS is_primary_key,
    c.name AS column_name,
    c."type" AS column_type,
    CASE WHEN c."notnull" THEN ''*'' ELSE '''' END AS not_null,
    COALESCE(c.dflt_value, '''') AS default_value,
    COALESCE((SELECT pfkl."table" || ''.'' || pfkl."to" FROM pragma_foreign_key_list(m.tbl_name) AS pfkl WHERE pfkl."from" = c.name), '''') as fk_refs,
    ROW_NUMBER() OVER (PARTITION BY m.tbl_name ORDER BY c.cid) AS row_num
  FROM sqlite_master m JOIN pragma_table_info(m.tbl_name) c ON 1=1
  WHERE m.type = ''table''
  ORDER BY table_name, row_num
),
Views AS (
  SELECT ''## Views '' AS markdown_output
  UNION ALL
  SELECT ''| View | Column | Type |'' AS markdown_output
  UNION ALL
  SELECT ''| ---- | ------ |----- |'' AS markdown_output
  UNION ALL
  SELECT ''| '' || tbl_name || '' | '' || c.name || '' | '' || c."type" || '' | ''
  FROM
    sqlite_master m,
    pragma_table_info(m.tbl_name) c
  WHERE
    m.type = ''view''
),
Indexes AS (
  SELECT ''## Indexes'' AS markdown_output
  UNION ALL
  SELECT ''| Table | Index | Columns |'' AS markdown_output
  UNION ALL
  SELECT ''| ----- | ----- | ------- |'' AS markdown_output
  UNION ALL
  SELECT ''| '' ||  m.name || '' | '' || il.name || '' | '' || group_concat(ii.name, '', '') || '' |'' AS markdown_output
  FROM sqlite_master as m,
    pragma_index_list(m.name) AS il,
    pragma_index_info(il.name) AS ii
  WHERE
    m.type = ''table''
  GROUP BY
    m.name,
    il.name
)
SELECT
    markdown_output AS info_schema_markdown
FROM
  (
    SELECT ''## Tables'' AS markdown_output
    UNION ALL
    SELECT
      CASE WHEN ti.row_num = 1 THEN ''
### `'' || ti.table_name || ''` Table
| PK | Column | Type | Req? | Default | References |
| -- | ------ | ---- | ---- | ------- | ---------- |
'' ||
        ''| '' || is_primary_key || '' | '' || ti.column_name || '' | '' || ti.column_type || '' | '' || ti.not_null || '' | '' || ti.default_value || '' | '' || ti.fk_refs || '' |''
      ELSE
        ''| '' || is_primary_key || '' | '' || ti.column_name || '' | '' || ti.column_type || '' | '' || ti.not_null || '' | '' || ti.default_value || '' | '' || ti.fk_refs || '' |''
      END
    FROM TableInfo ti
    UNION ALL SELECT ''''
    UNION ALL SELECT * FROM	Views
    UNION ALL SELECT ''''
    UNION ALL SELECT * FROM Indexes
);

-- :info_schema_markdown should be defined in the above query
SELECT ''text'' as component,
       ''Information Schema'' as title,
       :info_schema_markdown as contents_md', (CURRENT_TIMESTAMP)) ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP;
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('bad-item.sql', 'select ''alert'' as component,
                            ''MutationSqlNotebook.SQLPageSeedDML() issue'' as title,
                            ''SQLPageNotebook cell "bad-item.sql" did not return SQL (found: string)'' as description;', (CURRENT_TIMESTAMP)) ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP;
