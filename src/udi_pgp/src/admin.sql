  
CREATE TABLE IF NOT EXISTS "udi_pgp_supplier" (
    "udi_pgp_supplier_id" VARCHAR PRIMARY KEY NOT NULL,
    "type" TEXT NOT NULL,
    "mode" TEXT NOT NULL,
    "ssh_targets" TEXT CHECK(json_valid(ssh_targets) OR ssh_targets IS NULL),
    "auth" TEXT CHECK(json_valid(auth) OR auth IS NULL),
    "atc_file_path" TEXT,
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "udi_pgp_config" (
    "udi_pgp_config_id" VARCHAR PRIMARY KEY NOT NULL,
    "addr" TEXT NOT NULL,
    "health" TEXT,
    "metrics" TEXT,
    "config_ncl" TEXT,
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "udi_pgp_observe_query_exec" (
    "udi_pgp_observe_query_exec_id" UUID PRIMARY KEY NOT NULL,
    "query_text" TEXT NOT NULL,
    "exec_start_at" TIMESTAMPTZ NOT NULL,
    "exec_finish_at" TIMESTAMPTZ,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "exec_msg" TEXT[] NOT NULL,
    "exec_status" INTEGER NOT NULL,
    "governance" TEXT CHECK(json_valid(governance) OR governance IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT
);
CREATE TABLE IF NOT EXISTS "udi_pgp_set" (
    "udi_pgp_set_id" VARCHAR PRIMARY KEY NOT NULL,
    "query_text" TEXT NOT NULL,
    "generated_ncl" TEXT NOT NULL,
    "status" INTEGER NOT NULL,
    "status_text" TEXT,
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMPTZ,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMPTZ,
    "deleted_by" TEXT,
    "activity_log" TEXT
);

        