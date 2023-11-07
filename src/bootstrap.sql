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

-- store all SQL that is potentially reusable in the database
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKJ7KN4GQ4KXWM78WNA', 'SQL', 'ConstructionSqlNotebook', 'bootstrapDDL', NULL, 'CREATE TABLE IF NOT EXISTS "code_notebook_kernel" (
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


', 'ffa631cee6c3e2c6125596e43cf3f5513336e735', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKK447S61R2R55A10AT', 'SQL', 'ConstructionSqlNotebook', 'bootstrapSeedDML', NULL, 'storeNotebookCellsDML "bootstrapSeedDML" did not return SQL (found: object)', 'd5bcdf4a75be14925b1008a47362707c00cb8990', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKMY2A3QV5D3FS9DKDB', 'SQL', 'ConstructionSqlNotebook', 'initialDDL', NULL, 'CREATE TABLE IF NOT EXISTS "mime_type" (
    "mime_type_id" ULID PRIMARY KEY NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "file_extn" TEXT NOT NULL,
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("name", "file_extn")
);
CREATE TABLE IF NOT EXISTS "device" (
    "device_id" ULID PRIMARY KEY NOT NULL,
    "name" TEXT NOT NULL,
    "boundary" TEXT NOT NULL,
    "device_elaboration" TEXT CHECK(json_valid(device_elaboration) OR device_elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    UNIQUE("name", "boundary")
);
CREATE TABLE IF NOT EXISTS "fs_content_walk_session" (
    "fs_content_walk_session_id" ULID PRIMARY KEY NOT NULL,
    "device_id" ULID NOT NULL,
    "walk_started_at" TIMESTAMP NOT NULL,
    "walk_finished_at" TIMESTAMP,
    "max_fileio_read_bytes" INTEGER NOT NULL,
    "ignore_paths_regex" TEXT,
    "blobs_regex" TEXT,
    "digests_regex" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    UNIQUE("device_id", "created_at")
);
CREATE TABLE IF NOT EXISTS "fs_content_walk_path" (
    "fs_content_walk_path_id" ULID PRIMARY KEY NOT NULL,
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
    FOREIGN KEY("walk_session_id") REFERENCES "fs_content_walk_session"("fs_content_walk_session_id"),
    UNIQUE("walk_session_id", "root_path", "created_at")
);
CREATE TABLE IF NOT EXISTS "fs_content" (
    "fs_content_id" ULID PRIMARY KEY NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "walk_path_id" ULID NOT NULL,
    "file_path" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "file_bytes" INTEGER,
    "file_extn" TEXT,
    "file_mode" INTEGER,
    "file_mode_human" TEXT,
    "file_mtime" INTEGER,
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
    FOREIGN KEY("walk_session_id") REFERENCES "fs_content_walk_session"("fs_content_walk_session_id"),
    FOREIGN KEY("walk_path_id") REFERENCES "fs_content_walk_path"("fs_content_walk_path_id"),
    UNIQUE("content_digest", "file_path", "file_bytes", "file_mtime")
);
CREATE TABLE IF NOT EXISTS "fs_content_walk_path_entry" (
    "fs_content_walk_path_entry_id" ULID PRIMARY KEY NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "walk_path_id" ULID NOT NULL,
    "fs_content_id" ULID,
    "file_path_abs" TEXT NOT NULL,
    "file_path_rel_parent" TEXT NOT NULL,
    "file_path_rel" TEXT NOT NULL,
    "file_basename" TEXT NOT NULL,
    "file_extn" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT ''UNKNOWN'',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("walk_session_id") REFERENCES "fs_content_walk_session"("fs_content_walk_session_id"),
    FOREIGN KEY("walk_path_id") REFERENCES "fs_content_walk_path"("fs_content_walk_path_id"),
    FOREIGN KEY("fs_content_id") REFERENCES "fs_content"("fs_content_id")
);

CREATE INDEX IF NOT EXISTS "idx_mime_type__file_extn" ON "mime_type"("file_extn");
CREATE INDEX IF NOT EXISTS "idx_device__name" ON "device"("name");
CREATE INDEX IF NOT EXISTS "idx_fs_content_walk_path__walk_session_id__root_path" ON "fs_content_walk_path"("walk_session_id", "root_path");
CREATE INDEX IF NOT EXISTS "idx_fs_content__walk_session_id__file_path" ON "fs_content"("walk_session_id", "file_path");
CREATE INDEX IF NOT EXISTS "idx_fs_content_walk_path_entry__walk_session_id__file_path_abs" ON "fs_content_walk_path_entry"("walk_session_id", "file_path_abs");
', '519ab47e4f0b9c23745c138e5f9d6cc3ad195e0b', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKMQHKP98AXMGNANRHF', 'SQL', 'ConstructionSqlNotebook', 'fsContentWalkSessionStatsViewDDL', NULL, 'DROP VIEW IF EXISTS "fs_content_walk_session_stats";
CREATE VIEW IF NOT EXISTS "fs_content_walk_session_stats" AS
    WITH Summary AS (
        SELECT
            strftime(''%Y-%m-%d %H:%M:%S.%f'', fcws.walk_started_at) AS walk_datetime,
            strftime(''%f'', fcws.walk_finished_at - fcws.walk_started_at) AS walk_duration,
            COALESCE(fcwpe.file_extn, '''') AS file_extn,
            fcwp.root_path AS root_path,
            COUNT(fcwpe.fs_content_id) AS total_count,
            SUM(CASE WHEN fsc.content IS NOT NULL THEN 1 ELSE 0 END) AS with_content,
            SUM(CASE WHEN fsc.frontmatter IS NOT NULL THEN 1 ELSE 0 END) AS with_frontmatter,
            AVG(fsc.file_bytes) AS average_size,
            strftime(''%Y-%m-%d %H:%M:%S'', datetime(MIN(fsc.file_mtime), ''unixepoch'')) AS oldest,
            strftime(''%Y-%m-%d %H:%M:%S'', datetime(MAX(fsc.file_mtime), ''unixepoch'')) AS youngest
        FROM
            fs_content_walk_session AS fcws
        LEFT JOIN
            fs_content_walk_path AS fcwp ON fcws.fs_content_walk_session_id = fcwp.walk_session_id
        LEFT JOIN
            fs_content_walk_path_entry AS fcwpe ON fcwp.fs_content_walk_path_id = fcwpe.walk_path_id
        LEFT JOIN
            fs_content AS fsc ON fcwpe.fs_content_id = fsc.fs_content_id
        GROUP BY
            fcws.walk_started_at,
            fcws.walk_finished_at,
            fcwpe.file_extn,
            fcwp.root_path
        UNION ALL
        SELECT
            strftime(''%Y-%m-%d %H:%M:%S.%f'', fcws.walk_started_at) AS walk_datetime,
            strftime(''%f'', fcws.walk_finished_at - fcws.walk_started_at) AS walk_duration,
            ''ALL'' AS file_extn,
            fcwp.root_path AS root_path,
            COUNT(fcwpe.fs_content_id) AS total_count,
            SUM(CASE WHEN fsc.content IS NOT NULL THEN 1 ELSE 0 END) AS with_content,
            SUM(CASE WHEN fsc.frontmatter IS NOT NULL THEN 1 ELSE 0 END) AS with_frontmatter,
            AVG(fsc.file_bytes) AS average_size,
            strftime(''%Y-%m-%d %H:%M:%S'', datetime(MIN(fsc.file_mtime), ''unixepoch'')) AS oldest,
            strftime(''%Y-%m-%d %H:%M:%S'', datetime(MAX(fsc.file_mtime), ''unixepoch'')) AS youngest
        FROM
            fs_content_walk_session AS fcws
        LEFT JOIN
            fs_content_walk_path AS fcwp ON fcws.fs_content_walk_session_id = fcwp.walk_session_id
        LEFT JOIN
            fs_content_walk_path_entry AS fcwpe ON fcwp.fs_content_walk_path_id = fcwpe.walk_path_id
        LEFT JOIN
            fs_content AS fsc ON fcwpe.fs_content_id = fsc.fs_content_id
        GROUP BY
            fcws.walk_started_at,
            fcws.walk_finished_at,
            fcwp.root_path
    )
    SELECT
        walk_datetime,
        walk_duration,
        file_extn,
        root_path,
        total_count,
        with_content,
        with_frontmatter,
        CAST(ROUND(average_size) AS INTEGER) AS average_size,
        oldest,
        youngest
    FROM
        Summary
    ORDER BY
        walk_datetime,
        file_extn', '72f2d34facaa062f44d1645b0a148f379d890cb3', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKNYJ5MHC8AY6ACZGKD', 'SQL', 'MutationSqlNotebook', 'mimeTypesSeedDML', NULL, '-- loadExtnSQL not provided to load ''asg017/ulid/ulid0''
-- loadExtnSQL not provided to load ''asg017/http/http0''

-- This source: ''https://raw.githubusercontent.com/patrickmccallum/mimetype-io/master/src/mimeData.json''
-- has the given JSON structure:
-- [
--   {
--     "name": <MimeTypeName>,
--     "description": <Description>,
--     "types": [<Extension1>, <Extension2>, ...],
--     "alternatives": [<Alternative1>, <Alternative2>, ...]
--   },
--   ...
-- ]
-- The goal of the SQL query is to flatten this structure into rows where each row will
-- represent a single MIME type and one of its associated file extensions (from the ''types'' array).
-- The output will look like:
-- | name             | description  | type         | alternatives        |
-- |------------------|--------------|--------------|---------------------|
-- | <MimeTypeName>   | <Description>| <Extension1> | <AlternativesArray> |
-- | <MimeTypeName>   | <Description>| <Extension2> | <AlternativesArray> |
-- ... and so on for all MIME types and all extensions.
--
-- we take all those JSON array entries and insert them into our MIME Types table
INSERT or IGNORE INTO mime_type (mime_type_id, name, description, file_extn)
  SELECT ulid(),
         resource.value ->> ''$.name'' as name,
         resource.value ->> ''$.description'' as description,
         file_extns.value as file_extn
    FROM json_each(http_get_body(''https://raw.githubusercontent.com/patrickmccallum/mimetype-io/master/src/mimeData.json'')) as resource,
         json_each(resource.value, ''$.types'') AS file_extns;

INSERT INTO "mime_type" ("mime_type_id", "name", "description", "file_extn", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ((ulid()), ''application/typescript'', ''Typescript source'', ''.ts'', NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING;;', 'e59e07c573270b126e606cd1e82af8200f12e047', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKNMFXBFWETR9R263D6', 'SQL', 'QuerySqlNotebook', 'infoSchema', NULL, 'SELECT tbl_name AS table_name,
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
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKPHB5CQZW5SF110019', 'SQL', 'QuerySqlNotebook', 'infoSchemaOsQueryATCs', NULL, 'WITH table_columns AS (
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
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKPFBT362JYQK7QDFB7', 'SQL', 'QuerySqlNotebook', 'infoSchemaMarkdown', NULL, '-- TODO: https://github.com/lovasoa/SQLpage/discussions/109#discussioncomment-7359513
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
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKPWPSJ8C76DR6RKMBY', 'SQL', 'QuerySqlNotebook', 'htmlAnchors', NULL, '-- loadExtnSQL not provided to load ''asg017/html/html0''

-- find all HTML files in the fs_content table and return
-- each file and the anchors'' labels and hrefs in that file
-- TODO: create a table called fs_content_html_anchor to store this data after inserting it into fs_content
--       so that simple HTML lookups do not require the html0 extension to be loaded
WITH html_content AS (
  SELECT fs_content_id, content, content_digest, file_path, file_extn FROM fs_content WHERE file_extn = ''.html''
),
html AS (
  SELECT file_path,
         text as label,
         html_attribute_get(html, ''a'', ''href'') as href
    FROM html_content, html_each(html_content.content, ''a'')
)
SELECT * FROM html;
      ', '084f0754470f247d4a518fedebea35a90b498996', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKQESF6W6M3P27DEVT1', 'SQL', 'QuerySqlNotebook', 'htmlHeadMeta', NULL, '-- loadExtnSQL not provided to load ''asg017/html/html0''

-- find all HTML files in the fs_content table and return
-- each file and the <head><meta name="key" content="value"> pair
-- TODO: create a table called fs_content_html_head_meta to store this data after inserting it into fs_content
--       so that simple HTML lookups do not require the html0 extension to be loaded
WITH html_content AS (
  SELECT fs_content_id, content, content_digest, file_path, file_extn FROM fs_content WHERE file_extn = ''.html''
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
      ', '01d8897883e0ce27be0f5cb65ecc033b5129f2a4', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
            interpretable_code = EXCLUDED.interpretable_code,
            notebook_kernel_id = EXCLUDED.notebook_kernel_id,
            updated_at = CURRENT_TIMESTAMP,
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));
INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKQP0SBFQF9XAKNZTSZ', 'SQL', 'AssuranceSqlNotebook', 'test1', NULL, 'WITH test_plan AS (
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
            activity_log = json_insert(COALESCE(activity_log, '[]'), '$[' || json_array_length(COALESCE(activity_log, '[]')) || ']', json_object('code_notebook_cell_id', code_notebook_cell_id, 'notebook_kernel_id', notebook_kernel_id, 'notebook_name', notebook_name, 'cell_name', cell_name, 'cell_governance', cell_governance, 'interpretable_code', interpretable_code, 'interpretable_code_hash', interpretable_code_hash, 'description', description, 'arguments', arguments, 'created_at', created_at, 'created_by', created_by, 'updated_at', updated_at, 'updated_by', updated_by, 'deleted_at', deleted_at, 'deleted_by', deleted_by, 'activity_log', activity_log));;

INSERT INTO "code_notebook_cell" ("code_notebook_cell_id", "notebook_kernel_id", "notebook_name", "cell_name", "cell_governance", "interpretable_code", "interpretable_code_hash", "description", "arguments", "created_at", "created_by", "updated_at", "updated_by", "deleted_at", "deleted_by", "activity_log") VALUES ('01HEJJ4ZKRQM8MRPWQCJ2E7GRK', 'PlantUML', 'SqlNotebooksOrchestrator', 'infoSchemaDiagram', NULL, '@startuml IE
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

  entity "mime_type" as mime_type {
    * **mime_type_id**: ULID
    --
    * name: TEXT
    * description: TEXT
    * file_extn: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  entity "device" as device {
    * **device_id**: ULID
    --
    * name: TEXT
    * boundary: TEXT
      device_elaboration: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  entity "fs_content_walk_session" as fs_content_walk_session {
    * **fs_content_walk_session_id**: ULID
    --
    * device_id: ULID
    * walk_started_at: TIMESTAMP
      walk_finished_at: TIMESTAMP
    * max_fileio_read_bytes: INTEGER
      ignore_paths_regex: TEXT
      blobs_regex: TEXT
      digests_regex: TEXT
      elaboration: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  entity "fs_content_walk_path" as fs_content_walk_path {
    * **fs_content_walk_path_id**: ULID
    --
    * walk_session_id: ULID
    * root_path: TEXT
      elaboration: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  entity "fs_content" as fs_content {
    * **fs_content_id**: ULID
    --
    * walk_session_id: ULID
    * walk_path_id: ULID
    * file_path: TEXT
    * content_digest: TEXT
      content: BLOB
      file_bytes: INTEGER
      file_extn: TEXT
      file_mode: INTEGER
      file_mode_human: TEXT
      file_mtime: INTEGER
      content_fm_body_attrs: TEXT
      frontmatter: TEXT
      elaboration: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  entity "fs_content_walk_path_entry" as fs_content_walk_path_entry {
    * **fs_content_walk_path_entry_id**: ULID
    --
    * walk_session_id: ULID
    * walk_path_id: ULID
      fs_content_id: ULID
    * file_path_abs: TEXT
    * file_path_rel_parent: TEXT
    * file_path_rel: TEXT
    * file_basename: TEXT
      file_extn: TEXT
      elaboration: TEXT
      created_at: TIMESTAMP
      created_by: TEXT
      updated_at: TIMESTAMP
      updated_by: TEXT
      deleted_at: TIMESTAMP
      deleted_by: TEXT
      activity_log: TEXT
  }

  device |o..o{ fs_content_walk_session
  fs_content_walk_session |o..o{ fs_content_walk_path
  fs_content_walk_session |o..o{ fs_content
  fs_content_walk_path |o..o{ fs_content
  fs_content_walk_session |o..o{ fs_content_walk_path_entry
  fs_content_walk_path |o..o{ fs_content_walk_path_entry
  fs_content |o..o{ fs_content_walk_path_entry
@enduml', '8a77d74f1d94353e235e88c365b5ac7291fa8d2e', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) ON CONFLICT(notebook_name, cell_name, interpretable_code_hash) DO UPDATE SET
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
