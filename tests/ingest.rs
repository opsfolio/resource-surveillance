use std::{fs, path::Path, str};

use anyhow::anyhow;
use assert_cmd::Command;
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

    fs::remove_file(&db_path)?;

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
    ingest_fixtures()?;

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
fn test_md() -> anyhow::Result<()> {
    ingest_fixtures()?;

    let mut file_path = std::env::current_dir()?;
    file_path.push("support/test-fixtures/markdown-with-frontmatter.md");

    let rows = get_uniform_resource(&file_path)?;

    assert_eq!(rows.len(), 1);
    let resource = rows.get(0).unwrap();

    let content = fs::read(&file_path)?;
    let content = str::from_utf8(&content)?;
    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();
    // let frontmatter = extract_front_matter(content);

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
    ingest_fixtures()?;

    let mut db_path = std::env::current_dir()?;
    db_path.push("e2e-test.db");
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT file_extn FROM ur_ingest_session_fs_path_entry WHERE file_extn = '.txt';",
    )?;
    let ext: String = stmt.query_row([], |row| row.get(0))?;

    assert_eq!(ext, "xml".to_string());

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

    ingest_fixtures()?;
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
    fs::remove_file(&db_path)?;

    Ok(())
}
