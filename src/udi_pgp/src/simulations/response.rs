use pgwire::api::{
    results::{FieldFormat, FieldInfo},
    Type,
};

use crate::{
    error::{UdiPgpError, UdiPgpResult},
    simulations::{PgSettings, Setting, SELECT_PG_SETTINGS},
};

use super::{
    SELECT_CURRENT_SCHEMA, SELECT_PG_CATALOG, SELECT_TYPCATEGORY, SELECT_VERSION,
    SELECT_VERSION_DBEAVER, SELECT_VERSION_RESPONSE, SHOW_DATE_STYLE, SHOW_SEARCH_PATH,
};

use include_dir::{include_dir, Dir};

fn prepare_rows(settings: &mut Vec<String>, pg_settings: Vec<PgSettings>) -> Vec<&str> {
    for s in pg_settings {
        let setting_val = match &s.setting {
            Setting::Float(val) => val.to_string(),
            Setting::Str(val) => val.to_string(),
            Setting::Int(val) => val.to_string(),
        };
        settings.push(s.name.clone());
        settings.push(setting_val);
        settings.push(s.unit.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.category.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.short_desc.clone());
        settings.push(s.extra_desc.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.context.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.vartype.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(
            s.source
                .clone()
                .clone()
                .unwrap_or_else(|| "NULL".to_string()),
        );
        settings.push(s.min_val.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.max_val.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.enumvals.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.boot_val.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.reset_val.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.sourcefile.clone().unwrap_or_else(|| "NULL".to_string()));
        settings.push(s.sourceline.unwrap_or(0).to_string());
        settings.push(s.pending_restart.to_string());
    }

    settings.iter().map(AsRef::as_ref).collect()
}

pub fn driver_queries_response(query: &str) -> UdiPgpResult<(Vec<FieldInfo>, Vec<&str>)> {
    static RESPONSES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/simulations/responses");

    // println!("{}", query.trim());
    if query.trim().starts_with(
        "SELECT c.oid,
  n.nspname,
  c.relname",
    ) {
        let field_infos = vec![
            FieldInfo::new("oid".to_string(), None, None, Type::OID, FieldFormat::Text),
            FieldInfo::new(
                "nspname".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            ),
            FieldInfo::new(
                "relname".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            ),
        ];
        let rows = vec!["78", "public", "remote-supplier"];
        return Ok((field_infos, rows));
    }

    // https://www.postgresql.org/docs/current/catalog-pg-class.html
    if query.trim().starts_with("SELECT c.relchecks, c.relkind, c.relhasindex, c.relhasrules, c.relhastriggers, c.relrowsecurity, c.relforcerowsecurity,") {
        let field_infos = vec![
            FieldInfo::new("relchecks".to_string(), None, None, Type::INT2, FieldFormat::Text),
            FieldInfo::new(
                "relkind".to_string(),
                None,
                None,
                Type::CHAR,
                FieldFormat::Text,
            ),
            FieldInfo::new(
                "relhasindex".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "relhasrules".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "relhastriggers".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "relrowsecurity".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "relforcerowsecurity".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "relhasoids".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
             FieldInfo::new(
                "relispartition".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
             FieldInfo::new(
                "reltablespace".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
               FieldInfo::new(
                "reloftype".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
            FieldInfo::new(
                "relpersistence".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
            FieldInfo::new(
                "relreplident".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
            FieldInfo::new(
                "amname".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
        ];
        let rows = vec!["1", "r", "false", "false", "false", "false", "false", "false", "false", "0", "0", "p", "n", "meh"];
        return Ok((field_infos, rows))
    }

    if query.trim().starts_with("SELECT a.attname,
  pg_catalog.format_type(a.atttypid, a.atttypmod),") {
     let field_infos = vec![
            FieldInfo::new("attname".to_string(), None, None, Type::VARCHAR, FieldFormat::Text),
            FieldInfo::new(
                "".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "attnotnull".to_string(),
                None,
                None,
                Type::BOOL,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "attcollation".to_string(),
                None,
                None,
                Type::OID,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "attidentity".to_string(),
                None,
                None,
                Type::CHAR,
                FieldFormat::Text,
            ),
                FieldInfo::new(
                "attgenerated".to_string(),
                None,
                None,
                Type::CHAR,
                FieldFormat::Text,
            ),
        ];
        let rows = vec!["mehh", "", "", "false", "55", "d", "s"];
        return Ok((field_infos, rows))
  }

    if query.trim().starts_with("SELECT r.conname, pg_catalog.pg_get_constraintdef(r.oid, true)
FROM pg_catalog.pg_constraint r") {
     let field_infos = vec![
            FieldInfo::new("conname".to_string(), None, None, Type::VARCHAR, FieldFormat::Text),
            FieldInfo::new(
                "".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            ),
        ];
        let rows = vec!["mehh", "stub for pg_constraint"];
        return Ok((field_infos, rows))
  }

    
    match query {
        SHOW_SEARCH_PATH => Ok((
            vec![FieldInfo::new(
                "search_path".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            )],
            vec!["information_schema, public, '$user'"],
        )),
        SHOW_DATE_STYLE => Ok((
            vec![FieldInfo::new(
                "DateStyle".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            )],
            vec!["ISO, MDY"],
        )),
        SELECT_CURRENT_SCHEMA => Ok((
            vec![
                FieldInfo::new(
                    "current_schema".to_string(),
                    None,
                    None,
                    Type::VARCHAR,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "session_user".to_string(),
                    None,
                    None,
                    Type::VARCHAR,
                    FieldFormat::Text,
                ),
            ],
            vec!["public", "postgres"],
        )),
        SELECT_TYPCATEGORY => Ok((
            vec![FieldInfo::new(
                "typecategory".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            )],
            vec![""],
        )),
        SELECT_VERSION_DBEAVER => Ok((
            vec![FieldInfo::new(
                "version".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            )],
            vec![SELECT_VERSION_RESPONSE],
        )),
        SELECT_VERSION => Ok((
            vec![FieldInfo::new(
                "version".to_string(),
                None,
                None,
                Type::VARCHAR,
                FieldFormat::Text,
            )],
            vec![SELECT_VERSION_RESPONSE],
        )),
        SELECT_PG_CATALOG => {
            let schema = vec![
                FieldInfo::new("oid".to_string(), None, None, Type::OID, FieldFormat::Text),
                FieldInfo::new(
                    "datname".to_string(),
                    None,
                    None,
                    Type::VARCHAR,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datdba".to_string(),
                    None,
                    None,
                    Type::OID,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "encoding".to_string(),
                    None,
                    None,
                    Type::INT4,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datcollate".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datctype".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datistemplate".to_string(),
                    None,
                    None,
                    Type::BOOL,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datallowconn".to_string(),
                    None,
                    None,
                    Type::BOOL,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datconnlimit".to_string(),
                    None,
                    None,
                    Type::INT4,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datlastsysoid".to_string(),
                    None,
                    None,
                    Type::OID,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datfrozenxid".to_string(),
                    None,
                    None,
                    Type::XID,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datminmxid".to_string(),
                    None,
                    None,
                    Type::XID,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "dattablespace".to_string(),
                    None,
                    None,
                    Type::OID,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "datacl".to_string(),
                    None,
                    None,
                    Type::ACLITEM_ARRAY,
                    FieldFormat::Text,
                ),
            ];
            Ok((
                schema,
                vec![
                    "13726", "postgres", "10", "6", "C.UTF-8", "C.UTF-8", "false", "true", "-1",
                    "13725", "726", "1", "1663", "NULL",
                ],
            ))
        }
        SELECT_PG_SETTINGS => {
            let schema = vec![
                FieldInfo::new(
                    "name".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "setting".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "unit".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "category".to_string(),
                    None,
                    None,
                    Type::VARCHAR,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "short_desc".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "extra_desc".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "context".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "vartype".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "source".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "min_val".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "max_val".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "enumvals".to_string(),
                    None,
                    None,
                    Type::TEXT_ARRAY,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "boot_val".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "reset_val".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "sourcefile".to_string(),
                    None,
                    None,
                    Type::TEXT,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "sourceline".to_string(),
                    None,
                    None,
                    Type::INT4,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "pending_restart".to_string(),
                    None,
                    None,
                    Type::BOOL,
                    FieldFormat::Text,
                ),
            ];

            let pg_settings_file = RESPONSES_DIR.get_file("pg_settings.json").unwrap();
            let pg_settings = pg_settings_file.contents_utf8().unwrap();
            let pg_settings: Vec<PgSettings> =
                serde_json::from_str(pg_settings).map_err(UdiPgpError::JsonError)?;

            let _rows = prepare_rows(&mut vec![], pg_settings.clone());
            Ok((schema, vec![]))
        }
        _ => {
            let schema = vec![
                FieldInfo::new(
                    "current_schema".to_string(),
                    None,
                    None,
                    Type::VARCHAR,
                    FieldFormat::Text,
                ),
                FieldInfo::new(
                    "session_user".to_string(),
                    None,
                    None,
                    Type::VARCHAR,
                    FieldFormat::Text,
                ),
            ];
            let rows = vec!["public", "postgres"];
            Ok((schema, rows))
        }
    }
}
