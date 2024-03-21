use anyhow::{anyhow, Context};
use common::query_sql_rows;
use html_parser::Dom;
use rusqlite::{params, Connection, Result as RusqliteResult, ToSql};
use scraper::{Html, Selector};
use sha1::{Digest, Sha1};

use crate::{ingest::INS_UR_TRANSFORM_SQL, persist::DbConn};

query_sql_rows!(
    get_content_by_nature,
    "SELECT content, uniform_resource_id FROM uniform_resource WHERE nature = ?",
    nature: &str;
    content: String, uniform_resource_id: String
);

#[derive(Debug)]
/// A transformed content
pub struct TransformedContent {
    /// Uniform Resource ID
    pub ur_id: String,
    pub uri: String,
    pub content: Vec<serde_json::Value>,
}

/// A transformer trait that should be implemented by all transformers
pub trait Transformer {
    /// The file extension for the group of resources to be transformed
    fn nature(&self) -> &'static str;
    /// Returns a handle to the underlying DbConn
    fn db_path(&self) -> String;
    /// Fetches all the resources matching the specified extension from the RSSD.
    /// Returns the `uniform_resource_id` and the content.
    /// TODO: change the return type to a standard struct
    fn resources(&self) -> anyhow::Result<Vec<(String, String)>> {
        let conn = Connection::open(self.db_path()).with_context(|| {
            format!(
                "[Transformer Resources]: Failed to open SQLite database {}",
                self.db_path()
            )
        })?;

        let mut results: Vec<(String, String)> = Vec::new();

        let nature = self.nature();
        get_content_by_nature(
            &conn,
            |_row_index, content, uniform_resource_id| {
                // let content_string = String::from_utf8(content)
                //     .map_err(|err| rusqlite::Error::UserFunctionError(Box::new(err)))?;
                results.push((uniform_resource_id, content));
                Ok(())
            },
            nature,
        )?;
        Ok(results)
    }
    /// Implement specific resource transformation
    fn transform(&self) -> anyhow::Result<Vec<TransformedContent>>;
    /// Inserts the transformed resource into `uniform_resource_transform`
    fn insert(&self) -> anyhow::Result<()> {
        let db_path = self.db_path();
        let mut dbc = DbConn::new(&db_path, 0)
            .with_context(|| format!("[ingest_imap] SQLite transaction in {}", db_path))?;
        let tx = dbc
            .init(None)
            .with_context(|| "[ingest_imap] Failed to start a database transaction")?;
        {
            let mut stmt = tx.prepare(INS_UR_TRANSFORM_SQL)?;

            let transformed_resources = &self.transform()?;
            for tc in transformed_resources {
                let content = serde_json::to_string_pretty(&tc.content)?;
                let size = content.as_bytes().len();
                let hash = {
                    let mut hasher = Sha1::new();
                    hasher.update(content.as_bytes());
                    format!("{:x}", hasher.finalize())
                };
                let _ur_transform_id: String = stmt.query_row(
                    params![
                        tc.ur_id,
                        tc.uri,
                        // Since the transform is actually in json
                        "json",
                        hash,
                        content,
                        size,
                        None::<&String>,
                    ],
                    |row| row.get(0),
                )?;
            }
        }

        tx.commit()
            .with_context(|| "[transfrom resources] Failed to commit the transaction")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HtmlTransformer {
    /// The select query name is first, followed by the selector itself
    pub css_selectors: Vec<(String, String)>,
    /// The RSSD path.
    pub db_path: String,
}

// fetch all html/ext type from uniform resource by adding a function to the trait to do that.
// transform them by doing any transformation and return a string to be added to resource transfrom
// add an insert function to the trait to insert the result of the transform function into ur_transfrom

impl HtmlTransformer {
    pub fn new(css_select: Vec<String>, db_path: String) -> Self {
        let css_selectors = css_select
            .iter()
            .filter_map(|s| {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    eprintln!(
                        "Warning: Invalid css_select format '{}'. Expected format 'name:selector'.",
                        s
                    );
                    None
                }
            })
            .collect::<Vec<_>>();
        HtmlTransformer {
            css_selectors,
            db_path,
        }
    }

    fn convert_html_to_value(&self, html: &str) -> anyhow::Result<serde_json::Value> {
        let html = ammonia::clean(html);
        let parsed_html = Dom::parse(&html)
            .map_err(|err| anyhow!("Failed to parse HTML element.\nError: {err:#?}"))?;
        let json_string = parsed_html.to_json_pretty()?;

        let mut json_value: serde_json::Value = serde_json::from_str(&json_string)
            .map_err(|err| anyhow!("Failed to parse JSON string.\nError: {err:#?}"))?;

        if let Some(children) = json_value["children"].take().as_array_mut() {
            if !children.is_empty() {
                // Take the first element from the array
                let first_child = children.remove(0);
                Ok(first_child)
            } else {
                Err(anyhow!("No children found in the parsed HTML JSON."))
            }
        } else {
            Err(anyhow!(
                "Expected a 'children' array in the parsed HTML JSON."
            ))
        }
    }
}

impl Transformer for HtmlTransformer {
    fn nature(&self) -> &'static str {
        "html"
    }

    fn db_path(&self) -> String {
        self.db_path.clone()
    }

    fn transform(&self) -> anyhow::Result<std::vec::Vec<TransformedContent>> {
        let resources = self.resources()?;
        let mut tcs = Vec::new();
        for (ur_id, html) in resources {
            for (select_query_name, css_selector) in &self.css_selectors {
                let fragment = Html::parse_fragment(&html);
                let selector = Selector::parse(css_selector)
                    .map_err(|err| anyhow!("Failed to parse CSS selector.\nError: {err:#?}"))?;

                let elements_json_values = fragment
                    .select(&selector)
                    .map(|el| {
                        let element_html = el.html();
                        self.convert_html_to_value(&element_html)
                    })
                    .collect::<anyhow::Result<Vec<serde_json::Value>>>()?;

                let uri = format!("css-select:{}", select_query_name);
                tcs.push(TransformedContent {
                    ur_id: ur_id.clone(),
                    uri,
                    content: elements_json_values,
                });
            }
        }

        Ok(tcs)
    }
}
