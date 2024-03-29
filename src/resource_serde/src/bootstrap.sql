CREATE TABLE IF NOT EXISTS "assurance_schema" (
    "assurance_schema_id" VARCHAR PRIMARY KEY NOT NULL,
    "assurance_type" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "code_json" TEXT CHECK(json_valid(code_json) OR code_json IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "code_notebook_kernel" (
    "code_notebook_kernel_id" VARCHAR PRIMARY KEY NOT NULL,
    "kernel_name" TEXT NOT NULL,
    "description" TEXT,
    "mime_type" TEXT,
    "file_extn" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("kernel_name")
);
CREATE TABLE IF NOT EXISTS "code_notebook_cell" (
    "code_notebook_cell_id" VARCHAR PRIMARY KEY NOT NULL,
    "notebook_kernel_id" VARCHAR NOT NULL,
    "notebook_name" TEXT NOT NULL,
    "cell_name" TEXT NOT NULL,
    "cell_governance" TEXT CHECK(json_valid(cell_governance) OR cell_governance IS NULL),
    "interpretable_code" TEXT NOT NULL,
    "interpretable_code_hash" TEXT NOT NULL,
    "description" TEXT,
    "arguments" TEXT CHECK(json_valid(arguments) OR arguments IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("notebook_kernel_id") REFERENCES "code_notebook_kernel"("code_notebook_kernel_id"),
    UNIQUE("notebook_name", "cell_name", "interpretable_code_hash")
);
CREATE TABLE IF NOT EXISTS "code_notebook_state" (
    "code_notebook_state_id" VARCHAR PRIMARY KEY NOT NULL,
    "code_notebook_cell_id" VARCHAR NOT NULL,
    "from_state" TEXT NOT NULL,
    "to_state" TEXT NOT NULL,
    "transition_result" TEXT CHECK(json_valid(transition_result) OR transition_result IS NULL),
    "transition_reason" TEXT,
    "transitioned_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("code_notebook_cell_id") REFERENCES "code_notebook_cell"("code_notebook_cell_id"),
    UNIQUE("code_notebook_cell_id", "from_state", "to_state")
);


;

INSERT INTO "code_notebook_kernel" ("code_notebook_kernel_id", "kernel_name", "description", "mime_type", "file_extn", "elaboration", "governance", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('SQL', 'Dialect-independent ANSI SQL', NULL, 'application/sql', '.sql', NULL, NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(kernel_name) DO UPDATE SET mime_type = EXCLUDED.mime_type, file_extn = EXCLUDED.file_extn;
INSERT INTO "code_notebook_kernel" ("code_notebook_kernel_id", "kernel_name", "description", "mime_type", "file_extn", "elaboration", "governance", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('DenoTaskShell', 'Deno Task Shell', NULL, 'application/x-deno-task-sh', '.deno-task-sh', NULL, NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(kernel_name) DO UPDATE SET mime_type = EXCLUDED.mime_type, file_extn = EXCLUDED.file_extn;
INSERT INTO "code_notebook_kernel" ("code_notebook_kernel_id", "kernel_name", "description", "mime_type", "file_extn", "elaboration", "governance", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('PlantUML', 'PlantUML ER Diagram', NULL, 'text/vnd.plantuml', '.puml', NULL, NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(kernel_name) DO UPDATE SET mime_type = EXCLUDED.mime_type, file_extn = EXCLUDED.file_extn;
INSERT INTO "code_notebook_kernel" ("code_notebook_kernel_id", "kernel_name", "description", "mime_type", "file_extn", "elaboration", "governance", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('LLM Prompt', 'Large Lanugage Model (LLM) Prompt', NULL, 'text/vnd.netspective.llm-prompt', '.llm-prompt.txt', NULL, NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(kernel_name) DO UPDATE SET mime_type = EXCLUDED.mime_type, file_extn = EXCLUDED.file_extn;

-- store all SQL that is potentially reusable in the database
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'BootstrapSqlNotebook', 'bootstrapDDL', NULL, 'CREATE TABLE IF NOT EXISTS "assurance_schema" (
    "assurance_schema_id" VARCHAR PRIMARY KEY NOT NULL,
    "assurance_type" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "code_json" TEXT CHECK(json_valid(code_json) OR code_json IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "code_notebook_kernel" (
    "code_notebook_kernel_id" VARCHAR PRIMARY KEY NOT NULL,
    "kernel_name" TEXT NOT NULL,
    "description" TEXT,
    "mime_type" TEXT,
    "file_extn" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("kernel_name")
);
CREATE TABLE IF NOT EXISTS "code_notebook_cell" (
    "code_notebook_cell_id" VARCHAR PRIMARY KEY NOT NULL,
    "notebook_kernel_id" VARCHAR NOT NULL,
    "notebook_name" TEXT NOT NULL,
    "cell_name" TEXT NOT NULL,
    "cell_governance" TEXT CHECK(json_valid(cell_governance) OR cell_governance IS NULL),
    "interpretable_code" TEXT NOT NULL,
    "interpretable_code_hash" TEXT NOT NULL,
    "description" TEXT,
    "arguments" TEXT CHECK(json_valid(arguments) OR arguments IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("notebook_kernel_id") REFERENCES "code_notebook_kernel"("code_notebook_kernel_id"),
    UNIQUE("notebook_name", "cell_name", "interpretable_code_hash")
);
CREATE TABLE IF NOT EXISTS "code_notebook_state" (
    "code_notebook_state_id" VARCHAR PRIMARY KEY NOT NULL,
    "code_notebook_cell_id" VARCHAR NOT NULL,
    "from_state" TEXT NOT NULL,
    "to_state" TEXT NOT NULL,
    "transition_result" TEXT CHECK(json_valid(transition_result) OR transition_result IS NULL),
    "transition_reason" TEXT,
    "transitioned_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("code_notebook_cell_id") REFERENCES "code_notebook_cell"("code_notebook_cell_id"),
    UNIQUE("code_notebook_cell_id", "from_state", "to_state")
);


', '95906133bce514389f13fcc6d1cd72f4231155c6', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
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
    "device_id" VARCHAR PRIMARY KEY NOT NULL,
    "name" TEXT NOT NULL,
    "state" TEXT CHECK(json_valid(state)) NOT NULL,
    "boundary" TEXT NOT NULL,
    "segmentation" TEXT CHECK(json_valid(segmentation) OR segmentation IS NULL),
    "state_sysinfo" TEXT CHECK(json_valid(state_sysinfo) OR state_sysinfo IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("name", "state", "boundary")
);
CREATE TABLE IF NOT EXISTS "behavior" (
    "behavior_id" VARCHAR PRIMARY KEY NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "behavior_name" TEXT NOT NULL,
    "behavior_conf_json" TEXT CHECK(json_valid(behavior_conf_json)) NOT NULL,
    "assurance_schema_id" VARCHAR,
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("assurance_schema_id") REFERENCES "assurance_schema"("assurance_schema_id"),
    UNIQUE("device_id", "behavior_name")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_resource_path_match_rule" (
    "ur_ingest_resource_path_match_rule_id" VARCHAR PRIMARY KEY NOT NULL,
    "namespace" TEXT NOT NULL,
    "regex" TEXT NOT NULL,
    "flags" TEXT NOT NULL,
    "nature" TEXT,
    "priority" TEXT,
    "description" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("namespace", "regex")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_resource_path_rewrite_rule" (
    "ur_ingest_resource_path_rewrite_rule_id" VARCHAR PRIMARY KEY NOT NULL,
    "namespace" TEXT NOT NULL,
    "regex" TEXT NOT NULL,
    "replace" TEXT NOT NULL,
    "priority" TEXT,
    "description" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("namespace", "regex", "replace")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session" (
    "ur_ingest_session_id" VARCHAR PRIMARY KEY NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "behavior_id" VARCHAR,
    "behavior_json" TEXT CHECK(json_valid(behavior_json) OR behavior_json IS NULL),
    "ingest_started_at" TIMESTAMPTZ NOT NULL,
    "ingest_finished_at" TIMESTAMPTZ,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("behavior_id") REFERENCES "behavior"("behavior_id"),
    UNIQUE("device_id", "created_at")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_fs_path" (
    "ur_ingest_session_fs_path_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "root_path" TEXT NOT NULL,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    UNIQUE("ingest_session_id", "root_path", "created_at")
);
CREATE TABLE IF NOT EXISTS "uniform_resource" (
    "uniform_resource_id" VARCHAR PRIMARY KEY NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "ingest_fs_path_id" VARCHAR,
    "ingest_imap_acct_folder_id" VARCHAR,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "last_modified_at" TIMESTAMPTZ,
    "content_fm_body_attrs" TEXT CHECK(json_valid(content_fm_body_attrs) OR content_fm_body_attrs IS NULL),
    "frontmatter" TEXT CHECK(json_valid(frontmatter) OR frontmatter IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("ingest_fs_path_id") REFERENCES "ur_ingest_session_fs_path"("ur_ingest_session_fs_path_id"),
    FOREIGN KEY("ingest_imap_acct_folder_id") REFERENCES "ur_ingest_session_imap_acct_folder"("ur_ingest_session_imap_acct_folder_id"),
    UNIQUE("device_id", "content_digest", "uri", "size_bytes", "last_modified_at")
);
CREATE TABLE IF NOT EXISTS "uniform_resource_transform" (
    "uniform_resource_transform_id" VARCHAR PRIMARY KEY NOT NULL,
    "uniform_resource_id" VARCHAR NOT NULL,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id"),
    UNIQUE("uniform_resource_id", "content_digest", "nature", "size_bytes")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_fs_path_entry" (
    "ur_ingest_session_fs_path_entry_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "ingest_fs_path_id" VARCHAR NOT NULL,
    "uniform_resource_id" VARCHAR,
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
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("ingest_fs_path_id") REFERENCES "ur_ingest_session_fs_path"("ur_ingest_session_fs_path_id"),
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_task" (
    "ur_ingest_session_task_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "uniform_resource_id" VARCHAR,
    "captured_executable" TEXT CHECK(json_valid(captured_executable)) NOT NULL,
    "ur_status" TEXT,
    "ur_diagnostics" TEXT CHECK(json_valid(ur_diagnostics) OR ur_diagnostics IS NULL),
    "ur_transformations" TEXT CHECK(json_valid(ur_transformations) OR ur_transformations IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_imap_account" (
    "ur_ingest_session_imap_account_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "email" TEXT,
    "password" TEXT,
    "host" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    UNIQUE("ingest_session_id", "email")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_imap_acct_folder" (
    "ur_ingest_session_imap_acct_folder_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "ingest_account_id" VARCHAR NOT NULL,
    "folder_name" TEXT NOT NULL,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("ingest_account_id") REFERENCES "ur_ingest_session_imap_account"("ur_ingest_session_imap_account_id"),
    UNIQUE("ingest_account_id", "folder_name")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_imap_acct_folder_message" (
    "ur_ingest_session_imap_acct_folder_message_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "ingest_imap_acct_folder_id" VARCHAR NOT NULL,
    "uniform_resource_id" VARCHAR,
    "message" TEXT NOT NULL,
    "message_id" TEXT NOT NULL,
    "subject" TEXT NOT NULL,
    "from" TEXT NOT NULL,
    "cc" TEXT CHECK(json_valid(cc)) NOT NULL,
    "bcc" TEXT CHECK(json_valid(bcc)) NOT NULL,
    "email_references" TEXT CHECK(json_valid(email_references)) NOT NULL,
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("ingest_imap_acct_folder_id") REFERENCES "ur_ingest_session_imap_acct_folder"("ur_ingest_session_imap_acct_folder_id"),
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id"),
    UNIQUE("message", "message_id")
);

CREATE INDEX IF NOT EXISTS "idx_device__name__state" ON "device"("name", "state");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_fs_path__ingest_session_id__root_path" ON "ur_ingest_session_fs_path"("ingest_session_id", "root_path");
CREATE INDEX IF NOT EXISTS "idx_uniform_resource__device_id__uri" ON "uniform_resource"("device_id", "uri");
CREATE INDEX IF NOT EXISTS "idx_uniform_resource_transform__uniform_resource_id__content_digest" ON "uniform_resource_transform"("uniform_resource_id", "content_digest");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_fs_path_entry__ingest_session_id__file_path_abs" ON "ur_ingest_session_fs_path_entry"("ingest_session_id", "file_path_abs");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_task__ingest_session_id" ON "ur_ingest_session_task"("ingest_session_id");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_imap_acct_folder__ingest_session_id__folder_name" ON "ur_ingest_session_imap_acct_folder"("ingest_session_id", "folder_name");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_imap_acct_folder_message__ingest_session_id" ON "ur_ingest_session_imap_acct_folder_message"("ingest_session_id");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_imap_account__ingest_session_id__email" ON "ur_ingest_session_imap_account"("ingest_session_id", "email");
', '4cdfbc7f1483b3ac3d908cb75c939429f8db4384', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v001_seedDML', NULL, 'INSERT INTO "ur_ingest_resource_path_match_rule" ("ur_ingest_resource_path_match_rule_id", "namespace", "regex", "flags", "nature", "priority", "description", "elaboration", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), ''default'', ''/(\.git|node_modules)/'', ''IGNORE_RESOURCE'', NULL, NULL, ''Ignore any entry with `/.git/` or `/node_modules/` in the path.'', NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING;
INSERT INTO "ur_ingest_resource_path_match_rule" ("ur_ingest_resource_path_match_rule_id", "namespace", "regex", "flags", "nature", "priority", "description", "elaboration", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), ''default'', ''\.(?P<nature>md|mdx|html|json|jsonc|puml|txt|toml|yml|xml|tap)$'', ''CONTENT_ACQUIRABLE'', ''?P<nature>'', NULL, ''Ingest the content for md, mdx, html, json, jsonc, puml, txt, toml, and yml extensions. Assume the nature is the same as the extension.'', NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING;
INSERT INTO "ur_ingest_resource_path_match_rule" ("ur_ingest_resource_path_match_rule_id", "namespace", "regex", "flags", "nature", "priority", "description", "elaboration", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), ''default'', ''surveilr\[(?P<nature>[^\]]*)\]'', ''CAPTURABLE_EXECUTABLE'', ''?P<nature>'', NULL, ''Any entry with `surveilr-[XYZ]` in the path will be treated as a capturable executable extracting `XYZ` as the nature'', NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING;
INSERT INTO "ur_ingest_resource_path_match_rule" ("ur_ingest_resource_path_match_rule_id", "namespace", "regex", "flags", "nature", "priority", "description", "elaboration", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), ''default'', ''surveilr-SQL'', ''CAPTURABLE_EXECUTABLE | CAPTURABLE_SQL'', NULL, NULL, ''Any entry with surveilr-SQL in the path will be treated as a capturable SQL executable and allow execution of the SQL'', NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING;

INSERT INTO "ur_ingest_resource_path_rewrite_rule" ("ur_ingest_resource_path_rewrite_rule_id", "namespace", "regex", "replace", "priority", "description", "elaboration", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), ''default'', ''(\.plantuml)$'', ''.puml'', NULL, ''Treat .plantuml as .puml files'', NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING;
INSERT INTO "ur_ingest_resource_path_rewrite_rule" ("ur_ingest_resource_path_rewrite_rule_id", "namespace", "regex", "replace", "priority", "description", "elaboration", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), ''default'', ''(\.text)$'', ''.txt'', NULL, ''Treat .text as .txt files'', NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING;
INSERT INTO "ur_ingest_resource_path_rewrite_rule" ("ur_ingest_resource_path_rewrite_rule_id", "namespace", "regex", "replace", "priority", "description", "elaboration", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), ''default'', ''(\.yaml)$'', ''.yml'', NULL, ''Treat .yaml as .yml files'', NULL, (CURRENT_TIMESTAMP), NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING;
', '49ef5d2732c31bf079ab2548ec0e9f3fcb47b9d8', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v002_fsContentIngestSessionFilesStatsViewDDL', NULL, 'DROP VIEW IF EXISTS "ur_ingest_session_files_stats";
CREATE VIEW IF NOT EXISTS "ur_ingest_session_files_stats" AS
    WITH Summary AS (
        SELECT
            device.device_id AS device_id,
            ur_ingest_session.ur_ingest_session_id AS ingest_session_id,
            ur_ingest_session.ingest_started_at AS ingest_session_started_at,
            ur_ingest_session.ingest_finished_at AS ingest_session_finished_at,
            COALESCE(ur_ingest_session_fs_path_entry.file_extn, '''') AS file_extension,
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
        file_extension;', '9870d0c179334958ddda85827e4966b406c86e0c', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v002_fsContentIngestSessionFilesStatsLatestViewDDL', NULL, 'DROP VIEW IF EXISTS "ur_ingest_session_files_stats_latest";
CREATE VIEW IF NOT EXISTS "ur_ingest_session_files_stats_latest" AS
    SELECT iss.*
      FROM ur_ingest_session_files_stats AS iss
      JOIN (  SELECT ur_ingest_session.ur_ingest_session_id AS latest_session_id
                FROM ur_ingest_session
            ORDER BY ur_ingest_session.ingest_finished_at DESC
               LIMIT 1) AS latest
        ON iss.ingest_session_id = latest.latest_session_id;', 'f7a286b3b64881e069f05950d992a3b04af5c8f3', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v002_urIngestSessionTasksStatsViewDDL', NULL, 'DROP VIEW IF EXISTS "ur_ingest_session_tasks_stats";
CREATE VIEW IF NOT EXISTS "ur_ingest_session_tasks_stats" AS
      WITH Summary AS (
          SELECT
            device.device_id AS device_id,
            ur_ingest_session.ur_ingest_session_id AS ingest_session_id,
            ur_ingest_session.ingest_started_at AS ingest_session_started_at,
            ur_ingest_session.ingest_finished_at AS ingest_session_finished_at,
            COALESCE(ur_ingest_session_task.ur_status, ''Ok'') AS ur_status,
            COALESCE(uniform_resource.nature, ''UNKNOWN'') AS nature,
            COUNT(ur_ingest_session_task.uniform_resource_id) AS total_file_count,
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
            ur_ingest_session_task ON ur_ingest_session.ur_ingest_session_id = ur_ingest_session_task.ingest_session_id
        LEFT JOIN
            uniform_resource ON ur_ingest_session_task.uniform_resource_id = uniform_resource.uniform_resource_id
        GROUP BY
            device.device_id,
            ur_ingest_session.ur_ingest_session_id,
            ur_ingest_session.ingest_started_at,
            ur_ingest_session.ingest_finished_at,
            ur_ingest_session_task.captured_executable
    )
    SELECT
        device_id,
        ingest_session_id,
        ingest_session_started_at,
        ingest_session_finished_at,
        ur_status,
        nature,
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
        ur_status;', '1f7a7f2e454cf81922df584750d84a4d39a2381e', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v002_urIngestSessionTasksStatsLatestViewDDL', NULL, 'DROP VIEW IF EXISTS "ur_ingest_session_tasks_stats_latest";
CREATE VIEW IF NOT EXISTS "ur_ingest_session_tasks_stats_latest" AS
    SELECT iss.*
      FROM ur_ingest_session_tasks_stats AS iss
      JOIN (  SELECT ur_ingest_session.ur_ingest_session_id AS latest_session_id
                FROM ur_ingest_session
            ORDER BY ur_ingest_session.ingest_finished_at DESC
               LIMIT 1) AS latest
        ON iss.ingest_session_id = latest.latest_session_id;', '633501882d79e8bf255919bff540f9d0143489e3', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'SQL', 'ConstructionSqlNotebook', 'v002_urIngestSessionFileIssueViewDDL', NULL, 'DROP VIEW IF EXISTS "ur_ingest_session_file_issue";
CREATE VIEW IF NOT EXISTS "ur_ingest_session_file_issue" AS
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
             ufs.ur_diagnostics;', '99136e34dcf27424fa7b873b319ac5400786691e', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
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
    "assurance_schema_id" VARCHAR PRIMARY KEY NOT NULL,
    "assurance_type" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "code_json" TEXT CHECK(json_valid(code_json) OR code_json IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "code_notebook_kernel" (
    "code_notebook_kernel_id" VARCHAR PRIMARY KEY NOT NULL,
    "kernel_name" TEXT NOT NULL,
    "description" TEXT,
    "mime_type" TEXT,
    "file_extn" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("kernel_name")
);
CREATE TABLE IF NOT EXISTS "code_notebook_cell" (
    "code_notebook_cell_id" VARCHAR PRIMARY KEY NOT NULL,
    "notebook_kernel_id" VARCHAR NOT NULL,
    "notebook_name" TEXT NOT NULL,
    "cell_name" TEXT NOT NULL,
    "cell_governance" TEXT CHECK(json_valid(cell_governance) OR cell_governance IS NULL),
    "interpretable_code" TEXT NOT NULL,
    "interpretable_code_hash" TEXT NOT NULL,
    "description" TEXT,
    "arguments" TEXT CHECK(json_valid(arguments) OR arguments IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("notebook_kernel_id") REFERENCES "code_notebook_kernel"("code_notebook_kernel_id"),
    UNIQUE("notebook_name", "cell_name", "interpretable_code_hash")
);
CREATE TABLE IF NOT EXISTS "code_notebook_state" (
    "code_notebook_state_id" VARCHAR PRIMARY KEY NOT NULL,
    "code_notebook_cell_id" VARCHAR NOT NULL,
    "from_state" TEXT NOT NULL,
    "to_state" TEXT NOT NULL,
    "transition_result" TEXT CHECK(json_valid(transition_result) OR transition_result IS NULL),
    "transition_reason" TEXT,
    "transitioned_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("code_notebook_cell_id") REFERENCES "code_notebook_cell"("code_notebook_cell_id"),
    UNIQUE("code_notebook_cell_id", "from_state", "to_state")
);



      ', '85958d3630a5761a53dade44c67533e246de074e', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
                   interpretable_code = EXCLUDED.interpretable_code,
                   notebook_kernel_id = EXCLUDED.notebook_kernel_id,
                   updated_at = CURRENT_TIMESTAMP,
                   activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'LLM Prompt', 'LargeLanguageModelsPromptsNotebook', 'understand service schema', NULL, 'Understand the following structure of an SQLite database designed to store cybersecurity and compliance data for files in a file system.
The database is designed to store devices in the ''device'' table and entities called ''resources'' stored in the immutable append-only
''uniform_resource'' table. Each time files are "walked" they are stored in ingestion session and link back to ''uniform_resource''. Because all
tables are generally append only and immutable it means that the ingest_session_fs_path_entry table can be used for revision control
and historical tracking of file changes.

Use the following SQLite Schema to generate SQL queries that interact with these tables and once you understand them let me know so I can ask you for help:

CREATE TABLE IF NOT EXISTS "device" (
    "device_id" VARCHAR PRIMARY KEY NOT NULL,
    "name" TEXT NOT NULL,
    "state" TEXT CHECK(json_valid(state)) NOT NULL,
    "boundary" TEXT NOT NULL,
    "segmentation" TEXT CHECK(json_valid(segmentation) OR segmentation IS NULL),
    "state_sysinfo" TEXT CHECK(json_valid(state_sysinfo) OR state_sysinfo IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("name", "state", "boundary")
);
CREATE TABLE IF NOT EXISTS "behavior" (
    "behavior_id" VARCHAR PRIMARY KEY NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "behavior_name" TEXT NOT NULL,
    "behavior_conf_json" TEXT CHECK(json_valid(behavior_conf_json)) NOT NULL,
    "assurance_schema_id" VARCHAR,
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("assurance_schema_id") REFERENCES "assurance_schema"("assurance_schema_id"),
    UNIQUE("device_id", "behavior_name")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_resource_path_match_rule" (
    "ur_ingest_resource_path_match_rule_id" VARCHAR PRIMARY KEY NOT NULL,
    "namespace" TEXT NOT NULL,
    "regex" TEXT NOT NULL,
    "flags" TEXT NOT NULL,
    "nature" TEXT,
    "priority" TEXT,
    "description" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("namespace", "regex")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_resource_path_rewrite_rule" (
    "ur_ingest_resource_path_rewrite_rule_id" VARCHAR PRIMARY KEY NOT NULL,
    "namespace" TEXT NOT NULL,
    "regex" TEXT NOT NULL,
    "replace" TEXT NOT NULL,
    "priority" TEXT,
    "description" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("namespace", "regex", "replace")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session" (
    "ur_ingest_session_id" VARCHAR PRIMARY KEY NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "behavior_id" VARCHAR,
    "behavior_json" TEXT CHECK(json_valid(behavior_json) OR behavior_json IS NULL),
    "ingest_started_at" TIMESTAMPTZ NOT NULL,
    "ingest_finished_at" TIMESTAMPTZ,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("behavior_id") REFERENCES "behavior"("behavior_id"),
    UNIQUE("device_id", "created_at")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_fs_path" (
    "ur_ingest_session_fs_path_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "root_path" TEXT NOT NULL,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    UNIQUE("ingest_session_id", "root_path", "created_at")
);
CREATE TABLE IF NOT EXISTS "uniform_resource" (
    "uniform_resource_id" VARCHAR PRIMARY KEY NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "ingest_fs_path_id" VARCHAR,
    "ingest_imap_acct_folder_id" VARCHAR,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "last_modified_at" TIMESTAMPTZ,
    "content_fm_body_attrs" TEXT CHECK(json_valid(content_fm_body_attrs) OR content_fm_body_attrs IS NULL),
    "frontmatter" TEXT CHECK(json_valid(frontmatter) OR frontmatter IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("ingest_fs_path_id") REFERENCES "ur_ingest_session_fs_path"("ur_ingest_session_fs_path_id"),
    FOREIGN KEY("ingest_imap_acct_folder_id") REFERENCES "ur_ingest_session_imap_acct_folder"("ur_ingest_session_imap_acct_folder_id"),
    UNIQUE("device_id", "content_digest", "uri", "size_bytes", "last_modified_at")
);
CREATE TABLE IF NOT EXISTS "uniform_resource_transform" (
    "uniform_resource_transform_id" VARCHAR PRIMARY KEY NOT NULL,
    "uniform_resource_id" VARCHAR NOT NULL,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id"),
    UNIQUE("uniform_resource_id", "content_digest", "nature", "size_bytes")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_fs_path_entry" (
    "ur_ingest_session_fs_path_entry_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "ingest_fs_path_id" VARCHAR NOT NULL,
    "uniform_resource_id" VARCHAR,
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
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("ingest_fs_path_id") REFERENCES "ur_ingest_session_fs_path"("ur_ingest_session_fs_path_id"),
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_task" (
    "ur_ingest_session_task_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "uniform_resource_id" VARCHAR,
    "captured_executable" TEXT CHECK(json_valid(captured_executable)) NOT NULL,
    "ur_status" TEXT,
    "ur_diagnostics" TEXT CHECK(json_valid(ur_diagnostics) OR ur_diagnostics IS NULL),
    "ur_transformations" TEXT CHECK(json_valid(ur_transformations) OR ur_transformations IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_imap_account" (
    "ur_ingest_session_imap_account_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "email" TEXT,
    "password" TEXT,
    "host" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    UNIQUE("ingest_session_id", "email")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_imap_acct_folder" (
    "ur_ingest_session_imap_acct_folder_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "ingest_account_id" VARCHAR NOT NULL,
    "folder_name" TEXT NOT NULL,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("ingest_account_id") REFERENCES "ur_ingest_session_imap_account"("ur_ingest_session_imap_account_id"),
    UNIQUE("ingest_account_id", "folder_name")
);
CREATE TABLE IF NOT EXISTS "ur_ingest_session_imap_acct_folder_message" (
    "ur_ingest_session_imap_acct_folder_message_id" VARCHAR PRIMARY KEY NOT NULL,
    "ingest_session_id" VARCHAR NOT NULL,
    "ingest_imap_acct_folder_id" VARCHAR NOT NULL,
    "uniform_resource_id" VARCHAR,
    "message" TEXT NOT NULL,
    "message_id" TEXT NOT NULL,
    "subject" TEXT NOT NULL,
    "from" TEXT NOT NULL,
    "cc" TEXT CHECK(json_valid(cc)) NOT NULL,
    "bcc" TEXT CHECK(json_valid(bcc)) NOT NULL,
    "email_references" TEXT CHECK(json_valid(email_references)) NOT NULL,
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("ingest_session_id") REFERENCES "ur_ingest_session"("ur_ingest_session_id"),
    FOREIGN KEY("ingest_imap_acct_folder_id") REFERENCES "ur_ingest_session_imap_acct_folder"("ur_ingest_session_imap_acct_folder_id"),
    FOREIGN KEY("uniform_resource_id") REFERENCES "uniform_resource"("uniform_resource_id"),
    UNIQUE("message", "message_id")
);

CREATE INDEX IF NOT EXISTS "idx_device__name__state" ON "device"("name", "state");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_fs_path__ingest_session_id__root_path" ON "ur_ingest_session_fs_path"("ingest_session_id", "root_path");
CREATE INDEX IF NOT EXISTS "idx_uniform_resource__device_id__uri" ON "uniform_resource"("device_id", "uri");
CREATE INDEX IF NOT EXISTS "idx_uniform_resource_transform__uniform_resource_id__content_digest" ON "uniform_resource_transform"("uniform_resource_id", "content_digest");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_fs_path_entry__ingest_session_id__file_path_abs" ON "ur_ingest_session_fs_path_entry"("ingest_session_id", "file_path_abs");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_task__ingest_session_id" ON "ur_ingest_session_task"("ingest_session_id");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_imap_acct_folder__ingest_session_id__folder_name" ON "ur_ingest_session_imap_acct_folder"("ingest_session_id", "folder_name");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_imap_acct_folder_message__ingest_session_id" ON "ur_ingest_session_imap_acct_folder_message"("ingest_session_id");
CREATE INDEX IF NOT EXISTS "idx_ur_ingest_session_imap_account__ingest_session_id__email" ON "ur_ingest_session_imap_account"("ingest_session_id", "email");


DROP VIEW IF EXISTS "ur_ingest_session_files_stats";
CREATE VIEW IF NOT EXISTS "ur_ingest_session_files_stats" AS
    WITH Summary AS (
        SELECT
            device.device_id AS device_id,
            ur_ingest_session.ur_ingest_session_id AS ingest_session_id,
            ur_ingest_session.ingest_started_at AS ingest_session_started_at,
            ur_ingest_session.ingest_finished_at AS ingest_session_finished_at,
            COALESCE(ur_ingest_session_fs_path_entry.file_extn, '''') AS file_extension,
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
      ', 'b03283e4fd0b7f867d564c08481d2adaec5315cf', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
                   interpretable_code = EXCLUDED.interpretable_code,
                   notebook_kernel_id = EXCLUDED.notebook_kernel_id,
                   updated_at = CURRENT_TIMESTAMP,
                   activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));;

INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'PlantUML', 'SqlNotebooksOrchestrator', 'surveilrInfoSchemaDiagram', NULL, '@startuml surveilr-state
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
    * **device_id**: VARCHAR
    --
    * name: TEXT
    * state: TEXT
    * boundary: TEXT
      segmentation: TEXT
      state_sysinfo: TEXT
      elaboration: TEXT
    --
    behaviors: Behavior[]
    urIngestSessions: UrIngestSession[]
    uniformResources: UniformResource[]
  }

  entity "behavior" as behavior {
    * **behavior_id**: VARCHAR
    --
    * device_id: VARCHAR
    * behavior_name: TEXT
    * behavior_conf_json: TEXT
      assurance_schema_id: VARCHAR
      governance: TEXT
    --
    urIngestSessions: UrIngestSession[]
  }

  entity "ur_ingest_resource_path_match_rule" as ur_ingest_resource_path_match_rule {
    * **ur_ingest_resource_path_match_rule_id**: VARCHAR
    --
    * namespace: TEXT
    * regex: TEXT
    * flags: TEXT
      nature: TEXT
      priority: TEXT
      description: TEXT
      elaboration: TEXT
  }

  entity "ur_ingest_resource_path_rewrite_rule" as ur_ingest_resource_path_rewrite_rule {
    * **ur_ingest_resource_path_rewrite_rule_id**: VARCHAR
    --
    * namespace: TEXT
    * regex: TEXT
    * replace: TEXT
      priority: TEXT
      description: TEXT
      elaboration: TEXT
  }

  entity "ur_ingest_session" as ur_ingest_session {
    * **ur_ingest_session_id**: VARCHAR
    --
    * device_id: VARCHAR
      behavior_id: VARCHAR
      behavior_json: TEXT
    * ingest_started_at: TIMESTAMPTZ
      ingest_finished_at: TIMESTAMPTZ
      elaboration: TEXT
    --
    urIngestSessionFsPaths: UrIngestSessionFsPath[]
    uniformResources: UniformResource[]
    urIngestSessionFsPathEntrys: UrIngestSessionFsPathEntry[]
    urIngestSessionImapAccounts: UrIngestSessionImapAccount[]
    urIngestSessionImapAcctFolders: UrIngestSessionImapAcctFolder[]
    urIngestSessionImapAcctFolderMessages: UrIngestSessionImapAcctFolderMessage[]
  }

  entity "ur_ingest_session_fs_path" as ur_ingest_session_fs_path {
    * **ur_ingest_session_fs_path_id**: VARCHAR
    --
    * ingest_session_id: VARCHAR
    * root_path: TEXT
      elaboration: TEXT
    --
    urIngestSessionFsPathEntrys: UrIngestSessionFsPathEntry[]
  }

  entity "uniform_resource" as uniform_resource {
    * **uniform_resource_id**: VARCHAR
    --
    * device_id: VARCHAR
    * ingest_session_id: VARCHAR
      ingest_fs_path_id: VARCHAR
      ingest_imap_acct_folder_id: VARCHAR
    * uri: TEXT
    * content_digest: TEXT
      content: BLOB
      nature: TEXT
      size_bytes: INTEGER
      last_modified_at: TIMESTAMPTZ
      content_fm_body_attrs: TEXT
      frontmatter: TEXT
      elaboration: TEXT
    --
    uniformResourceTransforms: UniformResourceTransform[]
  }

  entity "uniform_resource_transform" as uniform_resource_transform {
    * **uniform_resource_transform_id**: VARCHAR
    --
    * uniform_resource_id: VARCHAR
    * uri: TEXT
    * content_digest: TEXT
      content: BLOB
      nature: TEXT
      size_bytes: INTEGER
      elaboration: TEXT
  }

  entity "ur_ingest_session_fs_path_entry" as ur_ingest_session_fs_path_entry {
    * **ur_ingest_session_fs_path_entry_id**: VARCHAR
    --
    * ingest_session_id: VARCHAR
    * ingest_fs_path_id: VARCHAR
      uniform_resource_id: VARCHAR
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

  entity "ur_ingest_session_task" as ur_ingest_session_task {
    * **ur_ingest_session_task_id**: VARCHAR
    --
    * ingest_session_id: VARCHAR
      uniform_resource_id: VARCHAR
    * captured_executable: TEXT
      ur_status: TEXT
      ur_diagnostics: TEXT
      ur_transformations: TEXT
      elaboration: TEXT
  }

  entity "ur_ingest_session_imap_account" as ur_ingest_session_imap_account {
    * **ur_ingest_session_imap_account_id**: VARCHAR
    --
    * ingest_session_id: VARCHAR
      email: TEXT
      password: TEXT
      host: TEXT
      elaboration: TEXT
    --
    urIngestSessionImapAcctFolders: UrIngestSessionImapAcctFolder[]
  }

  entity "ur_ingest_session_imap_acct_folder" as ur_ingest_session_imap_acct_folder {
    * **ur_ingest_session_imap_acct_folder_id**: VARCHAR
    --
    * ingest_session_id: VARCHAR
    * ingest_account_id: VARCHAR
    * folder_name: TEXT
      elaboration: TEXT
    --
    urIngestSessionImapAcctFolderMessages: UrIngestSessionImapAcctFolderMessage[]
  }

  entity "ur_ingest_session_imap_acct_folder_message" as ur_ingest_session_imap_acct_folder_message {
    * **ur_ingest_session_imap_acct_folder_message_id**: VARCHAR
    --
    * ingest_session_id: VARCHAR
    * ingest_imap_acct_folder_id: VARCHAR
      uniform_resource_id: VARCHAR
    * message: TEXT
    * message_id: TEXT
    * subject: TEXT
    * from: TEXT
    * cc: TEXT
    * bcc: TEXT
    * email_references: TEXT
  }

  device |o..o{ behavior
  device |o..o{ ur_ingest_session
  behavior |o..o{ ur_ingest_session
  ur_ingest_session |o..o{ ur_ingest_session_fs_path
  device |o..o{ uniform_resource
  ur_ingest_session |o..o{ uniform_resource
  ur_ingest_session_fs_path |o..o{ uniform_resource
  ur_ingest_session_imap_acct_folder |o..o{ uniform_resource
  uniform_resource |o..o{ uniform_resource_transform
  ur_ingest_session |o..o{ ur_ingest_session_fs_path_entry
  ur_ingest_session_fs_path |o..o{ ur_ingest_session_fs_path_entry
  uniform_resource |o..o{ ur_ingest_session_fs_path_entry
  ur_ingest_session |o..o{ ur_ingest_session_task
  uniform_resource |o..o{ ur_ingest_session_task
  ur_ingest_session |o..o{ ur_ingest_session_imap_account
  ur_ingest_session |o..o{ ur_ingest_session_imap_acct_folder
  ur_ingest_session_imap_account |o..o{ ur_ingest_session_imap_acct_folder
  ur_ingest_session |o..o{ ur_ingest_session_imap_acct_folder_message
  ur_ingest_session_imap_acct_folder |o..o{ ur_ingest_session_imap_acct_folder_message
  uniform_resource |o..o{ ur_ingest_session_imap_acct_folder_message
@enduml', '23f9c3c8a6b75ff4c471689c9851fb2b84da7f1c', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
             interpretable_code = EXCLUDED.interpretable_code,
             notebook_kernel_id = EXCLUDED.notebook_kernel_id,
             updated_at = CURRENT_TIMESTAMP,
             activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), 'PlantUML', 'SqlNotebooksOrchestrator', 'notebooksInfoSchemaDiagram', NULL, '@startuml surveilr-code-notebooks
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
    * **assurance_schema_id**: VARCHAR
    --
    * assurance_type: TEXT
    * code: TEXT
      code_json: TEXT
      governance: TEXT
  }

  entity "code_notebook_kernel" as code_notebook_kernel {
    * **code_notebook_kernel_id**: VARCHAR
    --
    * kernel_name: TEXT
      description: TEXT
      mime_type: TEXT
      file_extn: TEXT
      elaboration: TEXT
      governance: TEXT
    --
    codeNotebookCells: CodeNotebookCell[]
  }

  entity "code_notebook_cell" as code_notebook_cell {
    * **code_notebook_cell_id**: VARCHAR
    --
    * notebook_kernel_id: VARCHAR
    * notebook_name: TEXT
    * cell_name: TEXT
      cell_governance: TEXT
    * interpretable_code: TEXT
    * interpretable_code_hash: TEXT
      description: TEXT
      arguments: TEXT
  }

  entity "code_notebook_state" as code_notebook_state {
    * **code_notebook_state_id**: VARCHAR
    --
    * code_notebook_cell_id: VARCHAR
    * from_state: TEXT
    * to_state: TEXT
      transition_result: TEXT
      transition_reason: TEXT
      transitioned_at: TIMESTAMPTZ
      elaboration: TEXT
  }

  code_notebook_kernel |o..o{ code_notebook_cell
  code_notebook_cell |o..o{ code_notebook_state
@enduml', '84e0fc3aa026060b7e071785c89d02eaf87e6cbf', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
             interpretable_code = EXCLUDED.interpretable_code,
             notebook_kernel_id = EXCLUDED.notebook_kernel_id,
             updated_at = CURRENT_TIMESTAMP,
             activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
;

-- insert SQLPage content for diagnostics and web server
CREATE TABLE IF NOT EXISTS "sqlpage_files" (
    "path" VARCHAR PRIMARY KEY NOT NULL,
    "contents" TEXT NOT NULL,
    "last_modified" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('index.sql', 'SELECT
  ''list'' as component,
  ''Get started: where to go from here ?'' as title,
  ''Here are some useful links to get you started with SQLPage.'' as description;
SELECT ''Content Ingestion Session Statistics'' as title,
  ''ingest-session-stats.sql'' as link,
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
INSERT INTO "sqlpage_files" ("path", "contents", "last_modified") VALUES ('ingest-session-stats.sql', 'SELECT ''table'' as component, 1 as search, 1 as sort;
SELECT ingest_session_started_at, file_extn, total_count, with_content, with_frontmatter, average_size from ur_ingest_session_files_stats;', (CURRENT_TIMESTAMP)) ON CONFLICT(path) DO UPDATE SET contents = EXCLUDED.contents, last_modified = CURRENT_TIMESTAMP;
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
