use std::{fs, path::Path, str, sync::Mutex};

use anyhow::anyhow;
use assert_cmd::Command;
use lazy_static::lazy_static;
use rusqlite::Connection;

#[derive(Debug, Clone)]
struct UR {
    _id: String,
    _uri: String,
    content: String,
    nature: String,
    size_bytes: u64,
    front_matter: Option<String>,
}

fn ingest_fixtures() -> anyhow::Result<()> {
    let mut db_path = std::env::current_dir()?;
    db_path.push("e2e-test.db");
    if db_path.exists() {
        fs::remove_file(db_path)?;
    }

    let mut fixtures_dir = std::env::current_dir()?;
    fixtures_dir.push("support/test-fixtures");

    let mut cmd = Command::cargo_bin("surveilr")?;
    let output = cmd
        .args([
            "ingest",
            "files",
            "-d",
            "e2e-test.db",
            "-r",
            fixtures_dir.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("Command failed with exit status: {}", output.status);
        return Err(anyhow!("Command failed"));
    }

    Ok(())
}

lazy_static! {
    static ref INIT: Mutex<()> = {
        let _guard = Mutex::new(());
        ingest_fixtures().expect("Failed to ingest fixtures");
        _guard
    };
}

fn get_uniform_resource(file_path: &Path) -> anyhow::Result<Vec<UR>> {
    let mut db_path = std::env::current_dir()?;
    db_path.push("e2e-test.db");
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT u.uniform_resource_id, u.uri, u.content, u.nature, u.size_bytes, u.frontmatter, f.ur_ingest_session_fs_path_entry_id
FROM uniform_resource u
JOIN ur_ingest_session_fs_path_entry f ON u.uniform_resource_id = f.uniform_resource_id
WHERE u.uri = ?1;
",
    )?;

    let iter = stmt.query_map([file_path.to_str().unwrap()], |row| {
        Ok(UR {
            _id: row.get(0)?,        // uniform_resource_id
            _uri: row.get(1)?,       // uri
            content: row.get(2)?,    // content
            nature: row.get(3)?,     // nature
            size_bytes: row.get(4)?, // size_bytes
            front_matter: row.get(5)?,
        })
    })?;

    let results: Result<Vec<_>, _> = iter.collect();
    results.map_err(Into::into)
}

fn _extract_front_matter(markdown: &str) -> Option<String> {
    let parts: Vec<&str> = markdown.splitn(3, "---").collect();
    if parts.len() == 3 {
        serde_yaml::from_str::<serde_yaml::Value>(parts[1])
            .ok()
            .and_then(|yaml| serde_yaml::to_string(&yaml).ok())
    } else {
        None
    }
}

#[test]
fn test_plain_text() -> anyhow::Result<()> {
    let _lock = INIT.lock().unwrap();

    let mut file_path = std::env::current_dir()?;
    file_path.push("support/test-fixtures/plain-text.txt");

    let rows = get_uniform_resource(&file_path)?;

    assert_eq!(rows.len(), 1);
    let resource = rows.get(0).unwrap();

    let content = fs::read(&file_path)?;
    let content = str::from_utf8(&content)?;
    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();

    assert_eq!(resource.content, content);
    assert_eq!(resource.size_bytes, file_size);
    assert_eq!(resource.nature, "txt");

    Ok(())
}

#[test]
fn test_html() -> anyhow::Result<()> {
    let _lock = INIT.lock().unwrap();

    let mut file_path = std::env::current_dir()?;
    file_path.push("support/test-fixtures/plain.html");

    let rows = get_uniform_resource(&file_path)?;

    assert_eq!(rows.len(), 1);
    let resource = rows.get(0).unwrap();

    let content = fs::read(&file_path)?;
    let content = str::from_utf8(&content)?;
    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();

    assert_eq!(resource.content, content);
    assert_eq!(resource.size_bytes, file_size);
    assert_eq!(resource.nature, "html");

    Ok(())
}

#[test]
fn test_json() -> anyhow::Result<()> {
    let _lock = INIT.lock().unwrap();

    let mut file_path = std::env::current_dir()?;
    file_path.push("support/test-fixtures/table.json");

    let rows = get_uniform_resource(&file_path)?;

    assert_eq!(rows.len(), 1);
    let resource = rows.get(0).unwrap();

    let content = fs::read(&file_path)?;
    let content = str::from_utf8(&content)?;
    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();

    assert_eq!(resource.content, content);
    assert_eq!(resource.size_bytes, file_size);
    assert_eq!(resource.nature, "json");

    Ok(())
}

#[test]
fn test_capturable_exec() -> anyhow::Result<()> {
    let _lock = INIT.lock().unwrap();

    let mut file_path = std::env::current_dir()?;
    file_path.push("support/test-fixtures/capturable-executable.surveilr[json].sh");

    let rows = get_uniform_resource(&file_path)?;

    assert_eq!(rows.len(), 1);
    let resource = rows.get(0).unwrap();

    let content = fs::read(&file_path)?;
    let content = str::from_utf8(&content)?;
    // let metadata = fs::metadata(file_path)?;
    // let file_size = metadata.len();

    assert_ne!(resource.content, content);
    // assert_eq!(resource.size_bytes, file_size);
    assert_eq!(resource.nature, "json");
    assert_eq!(resource.content, "{ \"test\": \"JSON\" }\n");

    Ok(())
}

#[test]
fn test_md() -> anyhow::Result<()> {
    let _lock = INIT.lock().unwrap();

    let mut file_path = std::env::current_dir()?;
    file_path.push("support/test-fixtures/markdown-with-frontmatter.md");

    let rows = get_uniform_resource(&file_path)?;

    assert_eq!(rows.len(), 1);
    let resource = rows.get(0).unwrap();

    let content = fs::read(&file_path)?;
    let content = str::from_utf8(&content)?;
    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();

    assert_eq!(resource.content, content);
    assert_eq!(resource.size_bytes, file_size);
    assert_eq!(resource.nature, "md");

    let frontmatter = &resource.front_matter;
    assert!(frontmatter.is_some());
    let frontmatter = frontmatter.clone().unwrap();
    assert!(frontmatter.contains("Markdown with YAML Frontmatter Fixture"));

    Ok(())
}

#[test]
fn test_xml() -> anyhow::Result<()> {
    let _lock = INIT.lock().unwrap();

    let mut db_path = std::env::current_dir()?;
    db_path.push("e2e-test.db");
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT file_extn FROM ur_ingest_session_fs_path_entry WHERE file_extn = 'xml';",
    )?;
    let ext: String = stmt.query_row([], |row| row.get(0))?;

    assert_eq!(ext, "xml".to_string());

    Ok(())
}

#[test]
fn test_source_code() -> anyhow::Result<()> {
    let _lock = INIT.lock().unwrap();

    let mut db_path = std::env::current_dir()?;
    db_path.push("e2e-test.db");
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn
        .prepare("SELECT file_extn FROM ur_ingest_session_fs_path_entry WHERE file_extn = 'ts';")?;
    let ext: String = stmt.query_row([], |row| row.get(0))?;

    assert_eq!(ext, "ts".to_string());

    Ok(())
}

#[test]
fn test_image() -> anyhow::Result<()> {
    let _lock = INIT.lock().unwrap();

    let mut db_path = std::env::current_dir()?;
    db_path.push("e2e-test.db");
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT file_extn, ur_diagnostics FROM ur_ingest_session_fs_path_entry WHERE file_extn = 'png';",
    )?;
    let (ext, diagnostics): (String, String) =
        stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    assert_eq!(ext, "png".to_string());

    let ur_diagnostics: serde_json::Value = serde_json::from_str(&diagnostics)?;
    let message = ur_diagnostics["message"].as_str().unwrap_or_default();
    assert_eq!(message, "content supplier was not provided");

    Ok(())
}

#[test]
fn test_ingest_session() -> anyhow::Result<()> {
    fn count_files<P: AsRef<Path>>(path: P) -> anyhow::Result<usize> {
        let count = fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| entry.metadata().map(|m| m.is_file()).unwrap_or(false))
            .count();

        Ok(count)
    }

    let _lock = INIT.lock().unwrap();
    let mut curr_dir = std::env::current_dir()?;

    let mut db_path = curr_dir.clone();
    db_path.push("e2e-test.db");
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT ingest_session_id, COUNT(*) AS file_count FROM ur_ingest_session_fs_path_entry GROUP BY ingest_session_id;",
    )?;
    let (_, no_of_files): (String, u64) =
        stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    curr_dir.push("support/test-fixtures");
    // removes the "synthetic-tasks-via-stdin" as it has no extension and surveilr doesn't account for it.
    let expected = count_files(&curr_dir)? - 1;

    assert_eq!(no_of_files, expected as u64);
    Ok(())
}
