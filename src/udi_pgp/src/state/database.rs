use super::StateManager;

use crate::{config::UdiPgpConfig, observability::log_entry::QueryLogEntry};
use common::{execute_sql, execute_sql_no_args};
use rusqlite::{Connection, Result as RusqliteResult, ToSql};
use tracing::info;
use uuid::Uuid;

execute_sql_no_args!(clear_suppliers, "DELETE FROM udi_pgp_supplier");

execute_sql!(
    insert_supplier,
    "INSERT INTO udi_pgp_supplier (udi_pgp_supplier_id, type, mode, ssh_targets, auth, atc_file_path, governance, created_at, created_by, updated_at, updated_by, deleted_at, deleted_by, activity_log) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP, 'UNKNOWN', NULL, NULL, NULL, NULL, NULL)",
    udi_pgp_supplier_id: String,
    supplier_type: String,
    mode: String,
    ssh_targets: String,
    auth: String,
    atc_file_path: Option<String>,
    governance: Option<String>
);

execute_sql_no_args!(clear_udi_pgp_config, "DELETE FROM udi_pgp_config");

execute_sql!(
    insert_udi_pgp_config,
    "INSERT INTO udi_pgp_config (udi_pgp_config_id, addr, health, metrics, config_ncl, admin_db_path, surveilr_version, governance, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,?8, CURRENT_TIMESTAMP, 'UNKNOWN')",
    udi_pgp_config_id: String,
    addr: String,
    health: Option<String>,
    metrics: Option<String>,
    config_ncl: String,
    admin_db_path: String,
    surveilr_version: String,
    governance: Option<String>
);

execute_sql!(
    upsert_udi_pgp_observe_query_exec,
    "INSERT INTO udi_pgp_observe_query_exec (udi_pgp_observe_query_exec_id, query_text, exec_start_at, exec_finish_at, elaboration, exec_msg, exec_status, governance, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP, 'UNKNOWN')
     ON CONFLICT(udi_pgp_observe_query_exec_id) DO UPDATE SET
     query_text=excluded.query_text,
     exec_start_at=excluded.exec_start_at,
     exec_finish_at=excluded.exec_finish_at,
     elaboration=excluded.elaboration,
     exec_msg=excluded.exec_msg,
     exec_status=excluded.exec_status,
     governance=excluded.governance",
    udi_pgp_observe_query_exec_id: String,
    query_text: String,
    exec_start_at: String,
    exec_finish_at: Option<String>,
    elaboration: String,
    exec_msg: String,
    exec_status: u8,
    governance: Option<String>
);

execute_sql!(
    insert_into_udi_pgp_set,
    "INSERT INTO udi_pgp_set (udi_pgp_set_id, query_text, generated_ncl, diagnostics_file, diagnostics_file_content, status, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5, 0, CURRENT_TIMESTAMP, 'UNKNOWN')",
    udi_pgp_set_id: String,
    query_text: String,
    generated_ncl: String,
    diagnostics_file: String,
    diagnostics_file_content: String
);

execute_sql!(
    update_udi_pgp_set,
    "UPDATE udi_pgp_set SET status = ?2, status_text = ?3, updated_at = CURRENT_TIMESTAMP WHERE udi_pgp_set_id = ?1",
    udi_pgp_set_id: String,
    status: u8,
    status_text: String
);

impl StateManager {
    pub fn update_suppliers(&self, config: &UdiPgpConfig) {
        let conn = &self.conn;

        info!("Clearing suppliers from DB");
        clear_suppliers(conn).expect("Failed to delete all suppliers from DB");

        info!("Inserting suppliers into DB");
        for (id, supplier) in &config.suppliers {
            let ssh_targets_json =
                serde_json::to_string(&supplier.ssh_targets).unwrap_or("null".to_string());
            let auth_json = serde_json::to_string(&supplier.auth).unwrap_or("null".to_string());
            let atc_file_path = supplier.atc_file_path.clone();

            insert_supplier(
                conn,
                id.to_string(),
                supplier.supplier_type.to_string(),
                supplier.mode.to_string(),
                ssh_targets_json,
                auth_json,
                atc_file_path,
                None,
            )
            .expect("Failed to insert suppliers");
        }

        info!("Inserting suppliers into DB was succesful");
    }

    pub fn update_core(&self, config: &UdiPgpConfig) {
        let conn = &self.conn;

        info!("Clearing core config from DB");
        clear_udi_pgp_config(conn).expect("Failed to clear core config from DB");

        info!("Inserting new core config");
        let admindb_path = config.admin_state_fs_path.to_str().unwrap();

        insert_udi_pgp_config(
            conn,
            Uuid::new_v4().to_string(),
            config.addr().to_string(),
            config.health.map(|s| s.to_string()),
            config.metrics.map(|s| s.to_string()),
            "".to_string(),
            admindb_path.to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
            None,
        )
        .expect("Failed to insert into core db");

        info!("Successfully inserted new core config into DB");
    }

    pub fn insert_query_log(&self, entry: &QueryLogEntry) {
        let conn = &self.conn;
        info! {"Preparing to insert log"};

        let elaboration =
            serde_json::to_string_pretty(&entry.elaboration).unwrap_or("null".to_string());
        let exec_status: u8 = if entry.exec_msg.is_empty() { 0 } else { 1 };
        let exec_msg = serde_json::to_string_pretty(&entry.exec_msg).unwrap_or("null".to_string());

        upsert_udi_pgp_observe_query_exec(
            conn,
            entry.query_id.to_string(),
            entry.query_text.to_string(),
            entry.exec_start_at.clone().unwrap_or_default(),
            entry.exec_finish_at.clone(),
            elaboration,
            exec_msg,
            exec_status,
            None,
        )
        .expect("Failed to insert log");

        info! {"Inserted log successfully"};
    }

    pub fn create_udi_pgp_set_record(
        &self,
        id: String,
        query: String,
        ncl: String,
        diagnostics_file: String,
        content: String,
    ) {
        info!("Preparring to create SET query record");
        let conn = &self.conn;
        insert_into_udi_pgp_set(conn, id, query, ncl, diagnostics_file, content)
            .expect("Failed to create SET query record");
        info!("Created SET record successfully");
    }

    pub fn update_udi_pgp_set_record(&self, entry: &QueryLogEntry) {
        info!("Preparring to create SET query record");
        let conn = &self.conn;
        let exec_status: u8 = if entry.exec_msg.is_empty() { 0 } else { 1 };
        let exec_msg = serde_json::to_string_pretty(&entry.exec_msg).unwrap_or("null".to_string());

        update_udi_pgp_set(conn, entry.query_id.to_string(), exec_status, exec_msg)
            .expect("Failed to updated SET query record");
        info!("Updated SET record successfully");
    }
}
