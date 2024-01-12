use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::canonicalize;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use bitflags::bitflags;
use chrono::{DateTime, Utc};
use is_executable::IsExecutable;
use regex::Captures;
use regex::Regex;
use rusqlite::{Connection, Result as RusqliteResult};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha1::{Digest, Sha1};
use tracing::error;

use crate::frontmatter::frontmatter;
use crate::shell::*;

// See src/resources.states.puml for PlantUML specification of the state machine

pub trait BinaryContent {
    fn content_digest_hash(&self) -> &str;
    fn content_binary(&self) -> &Vec<u8>;
}

pub type FrontmatterComponents = (
    crate::frontmatter::FrontmatterNature,
    Option<String>,
    Result<JsonValue, Box<dyn Error>>,
    String,
);

pub trait TextContent {
    fn content_digest_hash(&self) -> &str;
    fn content_text(&self) -> &str;
    fn frontmatter(&self) -> FrontmatterComponents;
}

pub type BinaryContentSupplier = Box<dyn Fn() -> Result<Box<dyn BinaryContent>, Box<dyn Error>>>;
pub type TextContentSupplier = Box<dyn Fn() -> Result<Box<dyn TextContent>, Box<dyn Error>>>;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct EncounterableResourceFlags: u32 {
        const CONTENT_ACQUIRABLE    = 0b00000001;
        const IGNORE_RESOURCE       = EncounterableResourceFlags::CONTENT_ACQUIRABLE.bits() << 1;
        const CAPTURABLE_EXECUTABLE = EncounterableResourceFlags::IGNORE_RESOURCE.bits() << 1;
        const CAPTURABLE_SQL        = EncounterableResourceFlags::CAPTURABLE_EXECUTABLE.bits() << 1;

        // all the above are considered "common flags", this const is the "last" common
        const TERMINAL_COMMON       = EncounterableResourceFlags::CAPTURABLE_SQL.bits();

        // add any special ContentResource-only flags after this, starting with TERMINAL_COMMON
    }

    // EncounteredResourceFlags "inherits" flags from EncounterableResourceFlags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct EncounteredResourceFlags: u32 {
        const CONTENT_ACQUIRABLE    = EncounterableResourceFlags::CONTENT_ACQUIRABLE.bits();
        const IGNORE_RESOURCE       = EncounterableResourceFlags::IGNORE_RESOURCE.bits();
        const CAPTURABLE_EXECUTABLE = EncounterableResourceFlags::CAPTURABLE_EXECUTABLE.bits();
        const CAPTURABLE_SQL        = EncounterableResourceFlags::CAPTURABLE_SQL.bits();
        const TERMINAL_INHERITED    = EncounterableResourceFlags::TERMINAL_COMMON.bits();

        // these flags are not "common" and are specific to EncounteredResourceFlags
        const IS_FILE                  = EncounteredResourceFlags::TERMINAL_INHERITED.bits() << 1;
        const IS_DIRECTORY             = EncounteredResourceFlags::IS_FILE.bits() << 1;
        const IS_SYMLINK               = EncounteredResourceFlags::IS_DIRECTORY.bits() << 1;
    }

    // ContentResourceFlags "inherits" flags from EncounteredResourceFlags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct ContentResourceFlags: u32 {
        const CONTENT_ACQUIRABLE    = EncounteredResourceFlags::CONTENT_ACQUIRABLE.bits();
        const IGNORE_RESOURCE       = EncounteredResourceFlags::IGNORE_RESOURCE.bits();
        const CAPTURABLE_EXECUTABLE = EncounteredResourceFlags::CAPTURABLE_EXECUTABLE.bits();
        const CAPTURABLE_SQL        = EncounteredResourceFlags::CAPTURABLE_SQL.bits();
        const TERMINAL_INHERITED    = EncounteredResourceFlags::TERMINAL_INHERITED.bits();

        // add any special ContentResource-only flags after this, starting with TERMINAL_INHERITED
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePathRewriteRule {
    #[serde(with = "serde_regex")]
    pub regex: Regex,
    pub replace: String,
}

impl ResourcePathRewriteRule {
    pub fn _is_match(&self, text: &str) -> Option<String> {
        if let Some(caps) = self.regex.captures(text) {
            if let Some(nature) = caps.name("nature") {
                return Some(nature.as_str().to_string());
            }
        }
        None
    }

    pub fn rewritten_text(&self, text: &str) -> Option<String> {
        if let Some(_caps) = self.regex.captures(text) {
            let rewritten_text = self
                .regex
                .replace(text, |_caps: &Captures| self.replace.to_string());
            return Some(rewritten_text.to_string());
        }
        None
    }
}

// "?P<nature>" in the `nature` field means read it from the Regex via group capture
const PFRE_READ_NATURE_FROM_REGEX: &str = "?P<nature>";
const PFRE_READ_NATURE_FROM_REGEX_CAPTURE: &str = "nature";

const DEFAULT_IGNORE_PATHS_REGEX_PATTERNS: [&str; 1] = [r"/(\.git|node_modules)/"];
const DEFAULT_ACQUIRE_CONTENT_EXTNS_REGEX_PATTERNS: [&str; 1] =
    [r"\.(?P<nature>md|mdx|html|json|jsonc|puml|txt|toml|yml)$"];
const DEFAULT_CAPTURE_EXEC_REGEX_PATTERNS: [&str; 1] = [r"surveilr\[(?P<nature>[^\]]*)\]"];
const DEFAULT_CAPTURE_SQL_EXEC_REGEX_PATTERNS: [&str; 1] = [r"surveilr-SQL"];

// Rewrite patterns will look for a single capture group and replace it in the
// path (allows "rewriting" of extensions / nature to allow "aliases"). Rewritten
// extensions are only used for nature lookups, original text remains unchanged.
// Rewrite rules are best for cases where you want an extension to "act like"
// another extension.
const DEFAULT_REWRITE_NATURE_PATTERNS: [(&str, &str); 3] = [
    (r"(\.plantuml)$", ".puml"),
    (r"(\.text)$", ".txt"),
    (r"(\.yaml)$", ".yml"),
];

// this file is similar to .gitignore and, if it appears in a directory or
// parent, it allows `surveilr` to ignore globs specified within it
const SMART_IGNORE_CONF_FILES: [&str; 1] = [".surveilr_ignore"];

#[derive(Clone, Serialize, Deserialize)]
pub struct PersistableFlaggableRegEx {
    pub regex: String,          // untyped to make it easier to serialize/deserialize
    pub flags: String,          // untyped to make it easier to serialize/deserialize
    pub nature: Option<String>, // if this is ?P<nature> then we read nature from reg-ex otherwise it's forced
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EncounterableResourcePathRules {
    pub flaggables: Vec<PersistableFlaggableRegEx>,
    pub rewrite_nature_regexs: Vec<ResourcePathRewriteRule>,
    pub smart_ignore_conf_files: Vec<String>,
}

query_sql_rows_no_args!(
    ur_ingest_resource_path_match_rules_default,
    r"  SELECT regex, flags, nature, description
          FROM ur_ingest_resource_path_match_rule
         WHERE namespace = 'default'
      ORDER BY priority ";
    regex: String,
    flags: String,
    nature: Option<String>,
    description: String
);

query_sql_rows_no_args!(
    ur_ingest_resource_path_rewrite_rules_default,
    r"  SELECT regex, replace, description
          FROM ur_ingest_resource_path_rewrite_rule
         WHERE namespace = 'default'
      ORDER BY priority ";
    regex: String,
    replace: String,
    description: String
);

impl Default for EncounterableResourcePathRules {
    fn default() -> Self {
        let ignore = DEFAULT_IGNORE_PATHS_REGEX_PATTERNS.map(|p| PersistableFlaggableRegEx {
            regex: p.to_string(),
            flags: "IGNORE_RESOURCE".to_string(),
            nature: None,
        });
        let content_acquirable =
            DEFAULT_ACQUIRE_CONTENT_EXTNS_REGEX_PATTERNS.map(|p| PersistableFlaggableRegEx {
                regex: p.to_string(),
                flags: "CONTENT_ACQUIRABLE".to_string(),
                nature: Some(PFRE_READ_NATURE_FROM_REGEX.to_string()),
            });
        let capturable_executables =
            DEFAULT_CAPTURE_EXEC_REGEX_PATTERNS.map(|p| PersistableFlaggableRegEx {
                regex: p.to_string(),
                flags: "CAPTURABLE_EXECUTABLE".to_string(),
                nature: Some(PFRE_READ_NATURE_FROM_REGEX.to_string()),
            });
        let capturable_executables_sql =
            DEFAULT_CAPTURE_SQL_EXEC_REGEX_PATTERNS.map(|p| PersistableFlaggableRegEx {
                regex: p.to_string(),
                flags: "CAPTURABLE_EXECUTABLE | CAPTURABLE_SQL".to_string(),
                nature: None,
            });

        // using strings to set flags to show how to obtain rules from DB/external
        let flaggables_iter = ignore
            .into_iter()
            .chain(content_acquirable)
            .chain(capturable_executables)
            .chain(capturable_executables_sql);

        EncounterableResourcePathRules {
            flaggables: flaggables_iter.collect(),
            rewrite_nature_regexs: DEFAULT_REWRITE_NATURE_PATTERNS
                .map(|p| ResourcePathRewriteRule {
                    regex: Regex::new(p.0).unwrap(),
                    replace: p.1.to_string(),
                })
                .to_vec(),
            smart_ignore_conf_files: SMART_IGNORE_CONF_FILES.map(|s| s.to_string()).to_vec(),
        }
    }
}

impl EncounterableResourcePathRules {
    pub fn default_from_conn(conn: &Connection) -> rusqlite::Result<Self> {
        let mut flaggables: Vec<PersistableFlaggableRegEx> = vec![];
        ur_ingest_resource_path_match_rules_default(conn, |_, regex, flags, nature, _| {
            flaggables.push(PersistableFlaggableRegEx {
                regex,
                flags,
                nature,
            });
            Ok(())
        })?;

        let mut rewrite_nature_regexs: Vec<ResourcePathRewriteRule> = vec![];
        ur_ingest_resource_path_rewrite_rules_default(conn, |_, regex, replace, _| {
            rewrite_nature_regexs.push(ResourcePathRewriteRule {
                regex: regex::Regex::new(&regex).unwrap(),
                replace,
            });
            Ok(())
        })?;

        Ok(EncounterableResourcePathRules {
            flaggables,
            rewrite_nature_regexs,
            smart_ignore_conf_files: SMART_IGNORE_CONF_FILES.map(|s| s.to_string()).to_vec(),
        })
    }

    pub fn _from_json_text(json_text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_text)
    }

    pub fn _persistable_json_text(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone)]
pub struct EncounterableResourceClass {
    pub flags: EncounterableResourceFlags,
    pub nature: Option<String>,
}

pub trait EncounterableResourceUriClassifier {
    fn classify(&self, uri: &str, class: &mut EncounterableResourceClass) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlaggableRegEx {
    #[serde(with = "serde_regex")]
    pub regex: regex::Regex, // untyped to make it easier to serialize/deserialize
    pub flags: EncounterableResourceFlags, // untyped to make it easier to serialize/deserialize
    pub nature: Option<String>,            // either None, `?P<nature>` or the actual nature
}

impl FlaggableRegEx {
    pub fn from_persistable(pfre: &PersistableFlaggableRegEx) -> anyhow::Result<Self> {
        Ok(FlaggableRegEx {
            regex: regex::Regex::new(&pfre.regex)?,
            flags: bitflags::parser::from_str(&pfre.flags)
                .map_err(|e| anyhow::Error::msg(format!("{}", e)))?,
            nature: pfre.nature.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterableResourcePathClassifier {
    pub flaggables: Vec<FlaggableRegEx>,
    pub rewrite_path_regexs: Vec<ResourcePathRewriteRule>, // we need to capture `nature` so we loop through each one
    pub smart_ignore_conf_files: Vec<String>,
}

impl Default for EncounterableResourcePathClassifier {
    fn default() -> Self {
        let erpr = EncounterableResourcePathRules::default();
        EncounterableResourcePathClassifier::from_path_rules(erpr).unwrap()
    }
}

impl EncounterableResourcePathClassifier {
    pub fn from_path_rules(erpr: EncounterableResourcePathRules) -> anyhow::Result<Self> {
        let mut flaggables: Vec<FlaggableRegEx> = vec![];
        for f in &erpr.flaggables {
            flaggables.push(FlaggableRegEx::from_persistable(f)?)
        }

        let rewrite_nature_regexs = erpr.rewrite_nature_regexs.to_vec();
        Ok(EncounterableResourcePathClassifier {
            flaggables,
            rewrite_path_regexs: rewrite_nature_regexs,
            smart_ignore_conf_files: erpr.smart_ignore_conf_files.to_owned(),
        })
    }

    pub fn default_from_conn(conn: &Connection) -> anyhow::Result<Self> {
        let rules = EncounterableResourcePathRules::default_from_conn(conn)?;
        Self::from_path_rules(rules)
    }

    pub fn add_ignore_exact(&mut self, pattern: &str) {
        self.flaggables.push(FlaggableRegEx {
            regex: regex::Regex::new(format!("^{}$", regex::escape(pattern)).as_str()).unwrap(),
            flags: EncounterableResourceFlags::IGNORE_RESOURCE,
            nature: None,
        });
    }

    pub fn as_formatted_tables(&self) -> (comfy_table::Table, comfy_table::Table) {
        let mut flaggables: comfy_table::Table =
            crate::format::prepare_table(vec!["Regex", "Flags", "Nature"]);
        for f in &self.flaggables {
            flaggables.add_row(vec![
                f.regex.to_string(),
                format!("{:?}", f.flags),
                f.nature.clone().unwrap_or("".to_string()),
            ]);
        }

        let mut rewrite_path_regexs: comfy_table::Table =
            crate::format::prepare_table(vec!["Rewrite Regex", "Replace With"]);
        for rprr in &self.rewrite_path_regexs {
            rewrite_path_regexs.add_row(vec![rprr.regex.to_string(), rprr.replace.to_string()]);
        }

        (flaggables, rewrite_path_regexs)
    }
}

impl EncounterableResourceUriClassifier for EncounterableResourcePathClassifier {
    fn classify(&self, text: &str, class: &mut EncounterableResourceClass) -> bool {
        for rnr in &self.rewrite_path_regexs {
            if let Some(rewritten_text) = rnr.rewritten_text(text) {
                // since we've rewritten the text, now recursively determine class
                // using the new path/text
                return self.classify(&rewritten_text, class);
            }
        }

        for f in &self.flaggables {
            if let Some(potential_nature) = &f.nature {
                // if the nature is "?P<nature>" it means that we want to read nature from Regex
                if potential_nature == PFRE_READ_NATURE_FROM_REGEX {
                    if let Some(caps) = f.regex.captures(text) {
                        if let Some(nature) = caps.name(PFRE_READ_NATURE_FROM_REGEX_CAPTURE) {
                            class.flags.insert(f.flags);
                            class.nature = Some(nature.as_str().to_string());
                            return true;
                        }
                    }
                } else {
                    // Since nature is NOT "?P<nature>", we take the nature value literally
                    class.flags.insert(f.flags);
                    class.nature = Some(potential_nature.clone());
                    return true;
                }
            } else if f.regex.is_match(text) {
                class.flags.insert(f.flags);
                return true;
            }
        }

        false
    }
}

pub struct ContentResource {
    pub flags: ContentResourceFlags,
    pub uri: String,
    pub nature: Option<String>,
    pub size: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub content_binary_supplier: Option<BinaryContentSupplier>,
    pub content_text_supplier: Option<TextContentSupplier>,
}

pub struct CapturableExecResource<Resource> {
    pub resource: Resource,
    pub executable: CapturableExecutable,
}

pub struct PlainTextResource<Resource> {
    pub resource: Resource,
}

pub struct HtmlResource<Resource> {
    pub resource: Resource,
}

pub struct ImageResource<Resource> {
    pub resource: Resource,
}

pub enum JsonFormat {
    Json,
    JsonWithComments,
    Unknown,
}

pub struct JsonResource<Resource> {
    pub resource: Resource,
    pub format: JsonFormat,
}

pub enum JsonableTextSchema {
    TestAnythingProtocol,
    Toml,
    Yaml,
    Unknown,
}

pub struct JsonableTextResource<Resource> {
    pub resource: Resource,
    pub schema: JsonableTextSchema,
}

pub struct MarkdownResource<Resource> {
    pub resource: Resource,
}

pub enum SourceCodeInterpreter {
    TypeScript,
    JavaScript,
    Rust,
    PlantUml,
    Unknown,
}

pub struct SourceCodeResource<Resource> {
    pub resource: Resource,
    pub interpreter: SourceCodeInterpreter,
}

pub enum XmlSchema {
    Svg,
    Unknown,
}

pub struct XmlResource<Resource> {
    pub resource: Resource,
    pub schema: XmlSchema,
}

pub enum UniformResource<Resource> {
    CapturableExec(CapturableExecResource<Resource>),
    Html(HtmlResource<Resource>),
    Image(ImageResource<Resource>),
    Json(JsonResource<Resource>),
    JsonableText(JsonableTextResource<Resource>),
    Markdown(MarkdownResource<Resource>),
    PlainText(PlainTextResource<Resource>),
    SourceCode(SourceCodeResource<Resource>),
    Xml(XmlResource<Resource>),
    Unknown(Resource, Option<String>),
}

pub trait UniformResourceSupplier<Resource> {
    fn uniform_resource(
        &self,
        rs: Resource,
    ) -> Result<Box<UniformResource<Resource>>, Box<dyn Error>>;
}

pub trait UriNatureSupplier<Resource> {
    fn uri(&self) -> &String;
    fn nature(&self) -> &Option<String>;
}

impl UriNatureSupplier<ContentResource> for UniformResource<ContentResource> {
    fn uri(&self) -> &String {
        match self {
            UniformResource::CapturableExec(cer) => &cer.resource.uri,
            UniformResource::Html(html) => &html.resource.uri,
            UniformResource::Image(img) => &img.resource.uri,
            UniformResource::Json(json) => &json.resource.uri,
            UniformResource::JsonableText(json) => &json.resource.uri,
            UniformResource::Markdown(md) => &md.resource.uri,
            UniformResource::PlainText(txt) => &txt.resource.uri,
            UniformResource::SourceCode(sc) => &sc.resource.uri,
            UniformResource::Xml(xml) => &xml.resource.uri,
            UniformResource::Unknown(cr, _alternate) => &cr.uri,
        }
    }

    fn nature(&self) -> &Option<String> {
        match self {
            UniformResource::CapturableExec(cer) => &cer.resource.nature,
            UniformResource::Html(html) => &html.resource.nature,
            UniformResource::Image(img) => &img.resource.nature,
            UniformResource::Json(json) => &json.resource.nature,
            UniformResource::JsonableText(jsonable) => &jsonable.resource.nature,
            UniformResource::Markdown(md) => &md.resource.nature,
            UniformResource::PlainText(txt) => &txt.resource.nature,
            UniformResource::SourceCode(sc) => &sc.resource.nature,
            UniformResource::Xml(xml) => &xml.resource.nature,
            UniformResource::Unknown(_cr, _alternate) => &None::<String>,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceBinaryContent {
    pub hash: String,
    pub binary: Vec<u8>,
}

impl BinaryContent for ResourceBinaryContent {
    fn content_digest_hash(&self) -> &str {
        &self.hash
    }

    fn content_binary(&self) -> &Vec<u8> {
        &self.binary
    }
}

#[derive(Debug, Clone)]
pub struct ResourceTextContent {
    pub hash: String,
    pub text: String,
}

impl TextContent for ResourceTextContent {
    fn content_digest_hash(&self) -> &str {
        &self.hash
    }

    fn content_text(&self) -> &str {
        &self.text
    }

    fn frontmatter(&self) -> FrontmatterComponents {
        frontmatter(&self.text)
    }
}

#[derive(Debug)]
pub struct EncounteredResourceMetaData {
    pub flags: EncounteredResourceFlags,
    pub nature: Option<String>,
    pub file_size: u64,
    pub created_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>,
    pub last_modified_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>,
}

impl EncounteredResourceMetaData {
    pub fn from_fs_path(fs_path: &Path) -> anyhow::Result<EncounteredResourceMetaData> {
        let mut flags = EncounteredResourceFlags::empty();
        let file_size: u64;
        let created_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>;
        let last_modified_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>;

        match fs::metadata(fs_path) {
            Ok(metadata) => {
                flags.set(EncounteredResourceFlags::IS_FILE, metadata.is_file());
                flags.set(EncounteredResourceFlags::IS_DIRECTORY, metadata.is_dir());
                flags.set(EncounteredResourceFlags::IS_SYMLINK, metadata.is_symlink());
                file_size = metadata.len();
                created_at = metadata
                    .created()
                    .ok()
                    .map(chrono::DateTime::<chrono::Utc>::from);
                last_modified_at = metadata
                    .modified()
                    .ok()
                    .map(chrono::DateTime::<chrono::Utc>::from);
            }
            Err(err) => {
                let context = format!("ResourceContentMetaData::from_fs_path({:?})", fs_path,);
                return Err(anyhow::Error::new(err).context(context));
            }
        }

        let nature = fs_path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        Ok(EncounteredResourceMetaData {
            flags,
            nature,
            file_size,
            created_at,
            last_modified_at,
        })
    }

    pub fn from_vfs_path(vfs_path: &vfs::VfsPath) -> anyhow::Result<EncounteredResourceMetaData> {
        let mut flags = EncounteredResourceFlags::empty();

        let metadata = match vfs_path.metadata() {
            Ok(metadata) => {
                match metadata.file_type {
                    vfs::VfsFileType::File => {
                        flags.insert(EncounteredResourceFlags::IS_FILE);
                    }
                    vfs::VfsFileType::Directory => {
                        flags.insert(EncounteredResourceFlags::IS_DIRECTORY);
                    }
                };
                metadata
            }
            Err(err) => {
                let context = format!("ResourceContentMetaData::from_vfs_path({:?})", vfs_path);
                return Err(anyhow::Error::new(err).context(context));
            }
        };

        let nature = vfs_path
            .as_str()
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_string());

        Ok(EncounteredResourceMetaData {
            flags,
            nature,
            file_size: metadata.len,
            created_at: None,
            last_modified_at: None,
        })
    }
}

pub struct EncounteredResourceContentSuppliers {
    pub text: Option<TextContentSupplier>,
    pub binary: Option<BinaryContentSupplier>,
}

impl EncounteredResourceContentSuppliers {
    pub fn from_fs_path(
        fs_path: &Path,
        erc: &EncounterableResourceClass,
    ) -> EncounteredResourceContentSuppliers {
        let binary: Option<BinaryContentSupplier>;
        let text: Option<TextContentSupplier>;

        if erc
            .flags
            .contains(EncounterableResourceFlags::CONTENT_ACQUIRABLE)
        {
            let path_cbs = fs_path.to_string_lossy().to_string(); // Clone for the first closure
            binary = Some(Box::new(
                move || -> Result<Box<dyn BinaryContent>, Box<dyn Error>> {
                    let mut binary = Vec::new();
                    let mut file = fs::File::open(&path_cbs)?;
                    file.read_to_end(&mut binary)?;

                    let hash = {
                        let mut hasher = Sha1::new();
                        hasher.update(&binary);
                        format!("{:x}", hasher.finalize())
                    };

                    Ok(Box::new(ResourceBinaryContent { hash, binary }) as Box<dyn BinaryContent>)
                },
            ));

            let path_cts = fs_path.to_string_lossy().to_string(); // Clone for the second closure
            text = Some(Box::new(
                move || -> Result<Box<dyn TextContent>, Box<dyn Error>> {
                    let mut text = String::new();
                    let mut file = fs::File::open(&path_cts)?;
                    file.read_to_string(&mut text)?;

                    let hash = {
                        let mut hasher = Sha1::new();
                        hasher.update(&text);
                        format!("{:x}", hasher.finalize())
                    };

                    Ok(Box::new(ResourceTextContent { hash, text }) as Box<dyn TextContent>)
                },
            ));
        } else {
            binary = None;
            text = None;
        }

        EncounteredResourceContentSuppliers { binary, text }
    }

    pub fn from_vfs_path(
        vfs_path: &vfs::VfsPath,
        erc: &EncounterableResourceClass,
    ) -> EncounteredResourceContentSuppliers {
        let binary: Option<BinaryContentSupplier>;
        let text: Option<TextContentSupplier>;

        if erc
            .flags
            .contains(EncounterableResourceFlags::CONTENT_ACQUIRABLE)
        {
            let path_clone_cbs = vfs_path.clone();
            binary = Some(Box::new(
                move || -> Result<Box<dyn BinaryContent>, Box<dyn Error>> {
                    let mut binary = Vec::new();
                    let mut file = path_clone_cbs.open_file()?;
                    file.read_to_end(&mut binary)?;

                    let hash = {
                        let mut hasher = Sha1::new();
                        hasher.update(&binary);
                        format!("{:x}", hasher.finalize())
                    };

                    Ok(Box::new(ResourceBinaryContent { hash, binary }) as Box<dyn BinaryContent>)
                },
            ));

            let path_clone_cts = vfs_path.clone();
            text = Some(Box::new(
                move || -> Result<Box<dyn TextContent>, Box<dyn Error>> {
                    let mut text = String::new();
                    let mut file = path_clone_cts.open_file()?;
                    file.read_to_string(&mut text)?;

                    let hash = {
                        let mut hasher = Sha1::new();
                        hasher.update(&text);
                        format!("{:x}", hasher.finalize())
                    };

                    Ok(Box::new(ResourceTextContent { hash, text }) as Box<dyn TextContent>)
                },
            ));
        } else {
            text = None;
            binary = None;
        }

        EncounteredResourceContentSuppliers { text, binary }
    }
}

pub enum EncounterableResource {
    WalkDir(walkdir::DirEntry),
    SmartIgnore(ignore::DirEntry),
    Vfs(vfs::VfsPath),
    DenoTaskShellLine(String, Option<String>, String),
}

impl EncounterableResource {
    /// Parses a given string input as a JSON value and returns a DenoTaskShellLine.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice that represents either a JSON object or a plain text.
    ///
    /// # Returns
    ///
    /// DenoTaskShellLine:
    /// - The first string value found in the JSON object, or the entire input string if not a JSON object.
    /// - An `Option<String>` containing the key corresponding to the first string value, or `None` if the input is not a JSON object or doesn't contain a string value.
    /// - A string that is either `"json"` or the value of the `"nature"` key in the JSON object, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// let json_str = r#"{ "my_cmd_identity": "echo \"hello world\"", "nature": "text/plain" }"#;
    /// let result = dts_er(json_str);
    /// assert_eq!(result, ("echo \"hello world\"".to_string(), Some("my_cmd_identity".to_string()), "text/plain".to_string()));
    ///
    /// let non_json_str = "echo \"Hello, world!\"";
    /// let result = dts_er(non_json_str);
    /// assert_eq!(result, ("Hello, world!".to_string(), None, "json".to_string()));
    /// ```
    pub fn from_deno_task_shell_line(line: impl AsRef<str>) -> EncounterableResource {
        let default_nature = "json".to_string();
        let (commands, identity, nature) = match serde_json::from_str::<JsonValue>(line.as_ref()) {
            Ok(parsed) => {
                if let Some(obj) = parsed.as_object() {
                    let mut task: String = "no task found".to_string();
                    let mut identity: Option<String> = None;
                    let mut nature = default_nature.clone();
                    obj.iter()
                        .filter(|(_, v)| v.is_string())
                        .for_each(|(key, value)| {
                            if key == "nature" {
                                nature = JsonValue::as_str(value)
                                    .unwrap_or(default_nature.as_str())
                                    .to_string();
                            } else {
                                task = JsonValue::as_str(value)
                                    .unwrap_or(default_nature.as_str())
                                    .to_string();
                                identity = Some(key.to_owned());
                            }
                        });

                    (task, identity, nature)
                } else {
                    (line.as_ref().to_owned(), None, default_nature)
                }
            }
            Err(_) => (line.as_ref().to_owned(), None, default_nature),
        };
        EncounterableResource::DenoTaskShellLine(commands, identity, nature)
    }
}

pub enum EncounteredResource<T> {
    Ignored(String, EncounterableResourceClass),
    NotFound(String, EncounterableResourceClass),
    NotFile(String, EncounterableResourceClass),
    Resource(T, EncounterableResourceClass),
    CapturableExec(T, CapturableExecutable, EncounterableResourceClass),
}

impl ShellExecutive for EncounterableResource {
    fn execute(&self, std_in: ShellStdIn) -> anyhow::Result<ShellResult> {
        execute_subprocess(self.uri(), std_in)
    }
}

impl EncounterableResource {
    pub fn uri(&self) -> String {
        match self {
            EncounterableResource::WalkDir(de) => de.path().to_string_lossy().to_string(),
            EncounterableResource::SmartIgnore(de) => de.path().to_string_lossy().to_string(),
            EncounterableResource::Vfs(path) => path.as_str().to_string(),
            EncounterableResource::DenoTaskShellLine(line, identity, _) => {
                identity.to_owned().unwrap_or(line.as_str().to_string())
            }
        }
    }

    pub fn meta_data(&self) -> anyhow::Result<EncounteredResourceMetaData> {
        match self {
            EncounterableResource::WalkDir(de) => {
                EncounteredResourceMetaData::from_fs_path(de.path())
            }
            EncounterableResource::SmartIgnore(de) => {
                EncounteredResourceMetaData::from_fs_path(de.path())
            }
            EncounterableResource::Vfs(path) => EncounteredResourceMetaData::from_vfs_path(path),
            EncounterableResource::DenoTaskShellLine(_, _, nature) => {
                Ok(EncounteredResourceMetaData {
                    flags: EncounteredResourceFlags::empty(),
                    nature: Some(nature.clone()),
                    file_size: 0,
                    created_at: None,
                    last_modified_at: None,
                })
            }
        }
    }

    pub fn content_suppliers(
        &self,
        options: &EncounterableResourceClass,
    ) -> EncounteredResourceContentSuppliers {
        match self {
            EncounterableResource::WalkDir(de) => {
                EncounteredResourceContentSuppliers::from_fs_path(de.path(), options)
            }
            EncounterableResource::SmartIgnore(de) => {
                EncounteredResourceContentSuppliers::from_fs_path(de.path(), options)
            }
            EncounterableResource::Vfs(path) => {
                EncounteredResourceContentSuppliers::from_vfs_path(path, options)
            }
            EncounterableResource::DenoTaskShellLine(_, _, _) => {
                EncounteredResourceContentSuppliers {
                    text: None,
                    binary: None,
                }
            }
        }
    }

    pub fn encountered(
        &self,
        erc: &EncounterableResourceClass,
    ) -> EncounteredResource<ContentResource> {
        let uri = self.uri();

        if erc
            .flags
            .contains(EncounterableResourceFlags::IGNORE_RESOURCE)
        {
            return EncounteredResource::Ignored(uri, erc.to_owned());
        }

        let metadata = match self.meta_data() {
            Ok(metadata) => match self {
                EncounterableResource::WalkDir(_)
                | EncounterableResource::SmartIgnore(_)
                | EncounterableResource::Vfs(_) => {
                    if !metadata.flags.contains(EncounteredResourceFlags::IS_FILE) {
                        return EncounteredResource::NotFile(uri, erc.to_owned());
                    }
                    metadata
                }
                EncounterableResource::DenoTaskShellLine(_, _, _) => metadata,
            },
            Err(_) => return EncounteredResource::NotFound(uri, erc.to_owned()),
        };

        let content_suppliers = self.content_suppliers(erc);
        let nature: String;
        match &erc.nature {
            Some(classification_nature) => nature = classification_nature.to_owned(),
            None => match &metadata.nature {
                Some(md_nature) => nature = md_nature.to_owned(),
                None => nature = "json".to_string(),
            },
        }
        let cr: ContentResource = ContentResource {
            flags: ContentResourceFlags::from_bits_truncate(erc.flags.bits()),
            uri: uri.to_string(),
            nature: Some(nature.clone()),
            size: Some(metadata.file_size),
            created_at: metadata.created_at,
            last_modified_at: metadata.last_modified_at,
            content_binary_supplier: content_suppliers.binary,
            content_text_supplier: content_suppliers.text,
        };

        match self {
            EncounterableResource::WalkDir(_)
            | EncounterableResource::SmartIgnore(_)
            | EncounterableResource::Vfs(_) => {
                if erc
                    .flags
                    .contains(EncounterableResourceFlags::CAPTURABLE_EXECUTABLE)
                {
                    EncounteredResource::CapturableExec(
                        cr,
                        CapturableExecutable::from_encountered_content(self, erc),
                        erc.to_owned(),
                    )
                } else {
                    EncounteredResource::Resource(cr, erc.to_owned())
                }
            }
            EncounterableResource::DenoTaskShellLine(_, _, _) => {
                EncounteredResource::CapturableExec(
                    cr,
                    CapturableExecutable::from_encountered_content(self, erc),
                    erc.to_owned(),
                )
            }
        }
    }
}

pub enum CapturableExecutable {
    UriShellExecutive(Box<dyn ShellExecutive>, String, String, bool),
    RequestedButNotExecutable(String),
}

impl CapturableExecutable {
    pub fn from_encountered_content(
        er: &EncounterableResource,
        erc: &EncounterableResourceClass,
    ) -> CapturableExecutable {
        match er {
            EncounterableResource::WalkDir(de) => {
                CapturableExecutable::from_executable_file_path(de.path(), erc)
            }
            EncounterableResource::SmartIgnore(de) => {
                CapturableExecutable::from_executable_file_path(de.path(), erc)
            }
            EncounterableResource::Vfs(path) => {
                CapturableExecutable::from_executable_file_uri(path.as_str(), erc)
            }
            EncounterableResource::DenoTaskShellLine(line, identity, nature) => {
                CapturableExecutable::UriShellExecutive(
                    Box::new(DenoTaskShellExecutive::new(
                        line.clone(),
                        identity.to_owned(),
                    )),
                    line.clone(),
                    nature.to_string(),
                    erc.flags
                        .contains(EncounterableResourceFlags::CAPTURABLE_SQL),
                )
            }
        }
    }

    // check if URI is executable based only on the filename pattern
    pub fn from_executable_file_uri(
        uri: &str,
        erc: &EncounterableResourceClass,
    ) -> CapturableExecutable {
        let executable_file_uri = uri.to_string();
        CapturableExecutable::UriShellExecutive(
            Box::new(executable_file_uri.clone()), // String has the `ShellExecutive` trait
            executable_file_uri,
            erc.nature.clone().unwrap_or("?nature".to_string()),
            erc.flags
                .contains(EncounterableResourceFlags::CAPTURABLE_SQL),
        )
    }

    // check if URI is executable based the filename pattern first, then physical FS validation of execute permission
    pub fn from_executable_file_path(
        path: &std::path::Path,
        erc: &EncounterableResourceClass,
    ) -> CapturableExecutable {
        if path.is_executable() {
            CapturableExecutable::from_executable_file_uri(path.to_str().unwrap(), erc)
        } else {
            CapturableExecutable::RequestedButNotExecutable(path.to_string_lossy().to_string())
        }
    }

    pub fn uri(&self) -> &str {
        match self {
            CapturableExecutable::UriShellExecutive(_, uri, _, _)
            | CapturableExecutable::RequestedButNotExecutable(uri) => uri.as_str(),
        }
    }

    pub fn executed_result_as_text(
        &self,
        std_in: ShellStdIn,
    ) -> anyhow::Result<(String, String, bool), serde_json::Value> {
        match self {
            CapturableExecutable::UriShellExecutive(
                executive,
                interpretable_code,
                nature,
                is_batched_sql,
            ) => match executive.execute(std_in) {
                Ok(shell_result) => {
                    if shell_result.success() {
                        Ok((shell_result.stdout, nature.clone(), *is_batched_sql))
                    } else {
                        Err(serde_json::json!({
                            "src": self.uri(),
                            "interpretable-code": interpretable_code,
                            "issue": "[CapturableExecutable::TextFromExecutableUri.executed_text] invalid exit status",
                            "remediation": "ensure that executable is called with proper arguments and input formats",
                            "nature": nature,
                            "exit-status": format!("{:?}", shell_result.status),
                            "stdout": shell_result.stdout,
                            "stderr": shell_result.stderr
                        }))
                    }
                }
                Err(err) => Err(serde_json::json!({
                    "src": self.uri(),
                    "interpretable-code": interpretable_code,
                    "issue": "[CapturableExecutable::TextFromExecutableUri.executed_text] execution error",
                    "rust-err": format!("{:?}", err),
                    "nature": nature,
                })),
            },
            CapturableExecutable::RequestedButNotExecutable(src) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNotExecutable.executed_sql] executable permissions not set",
                "remediation": "make sure that script has executable permissions set",
            })),
        }
    }

    pub fn executed_result_as_json(
        &self,
        std_in: ShellStdIn,
    ) -> anyhow::Result<(serde_json::Value, String, bool), serde_json::Value> {
        match self {
            CapturableExecutable::UriShellExecutive(
                executive,
                interpretable_code,
                nature,
                is_batched_sql,
            ) => match executive.execute(std_in) {
                Ok(shell_result) => {
                    if shell_result.success() {
                        let captured_text = shell_result.stdout;
                        let value: serde_json::Result<serde_json::Value> =
                            serde_json::from_str(&captured_text);
                        match value {
                            Ok(value) => Ok((value, nature.clone(), *is_batched_sql)),
                            Err(_) => Err(serde_json::json!({
                                "src": self.uri(),
                                "interpretable-code": interpretable_code,
                                "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_json] unable to deserialize JSON",
                                "remediation": "ensure that executable is emitting JSON (e.g. `--json`)",
                                "nature": nature,
                                "is-batched-sql": is_batched_sql,
                                "stdout": captured_text,
                                "exit-status": format!("{:?}", shell_result.status),
                                "stderr": shell_result.stderr
                            })),
                        }
                    } else {
                        Err(serde_json::json!({
                            "src": self.uri(),
                            "interpretable-code": interpretable_code,
                            "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_json] invalid exit status",
                            "remediation": "ensure that executable is called with proper arguments and input formats",
                            "nature": nature,
                            "is-batched-sql": is_batched_sql,
                            "exit-status": format!("{:?}", shell_result.status),
                            "stderr": shell_result.stderr
                        }))
                    }
                }
                Err(err) => Err(serde_json::json!({
                    "src": self.uri(),
                    "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_json] execution error",
                    "rust-err": format!("{:?}", err),
                    "nature": nature,
                    "is-batched-sql": is_batched_sql,
                })),
            },
            CapturableExecutable::RequestedButNotExecutable(src) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNotExecutable.executed_result_as_json] executable permissions not set",
                "remediation": "make sure that script has executable permissions set",
            })),
        }
    }

    pub fn executed_result_as_sql(
        &self,
        std_in: ShellStdIn,
    ) -> anyhow::Result<(String, String), serde_json::Value> {
        match self {
            CapturableExecutable::UriShellExecutive(
                executive,
                interpretable_code,
                nature,
                is_batched_sql,
            ) => {
                if *is_batched_sql {
                    match executive.execute(std_in) {
                        Ok(shell_result) => {
                            if shell_result.status.success() {
                                Ok((shell_result.stdout, nature.clone()))
                            } else {
                                Err(serde_json::json!({
                                    "src": self.uri(),
                                    "interpretable-code": interpretable_code,
                                    "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_sql] invalid exit status",
                                    "remediation": "ensure that executable is called with proper arguments and input formats",
                                    "nature": nature,
                                    "exit-status": format!("{:?}", shell_result.status),
                                    "stdout": shell_result.stdout,
                                    "stderr": shell_result.stderr
                                }))
                            }
                        }
                        Err(err) => Err(serde_json::json!({
                            "src": self.uri(),
                            "interpretable-code": interpretable_code,
                            "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_sql] execution error",
                            "rust-err": format!("{:?}", err),
                            "nature": nature,
                        })),
                    }
                } else {
                    Err(serde_json::json!({
                        "src": self.uri(),
                        "interpretable-code": interpretable_code,
                        "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_sql] is not classified as batch SQL",
                        "nature": nature,
                    }))
                }
            }
            CapturableExecutable::RequestedButNotExecutable(src) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNotExecutable.executed_result_as_sql] executable permissions not set",
                "remediation": "make sure that script has executable permissions set",
            })),
        }
    }
}

pub struct ResourcesCollection {
    pub encounterable: Vec<EncounterableResource>,
    pub classifier: EncounterableResourcePathClassifier,
    pub nature_aliases: Option<HashMap<String, String>>,
}

impl ResourcesCollection {
    pub fn new(
        encounterable: Vec<EncounterableResource>,
        classifier: &EncounterableResourcePathClassifier,
        nature_aliases: Option<HashMap<String, String>>,
    ) -> ResourcesCollection {
        ResourcesCollection {
            encounterable,
            classifier: classifier.clone(),
            nature_aliases: nature_aliases.clone(),
        }
    }

    // create a physical file system mapped via VFS, mainly for testing and experimental use
    pub fn from_vfs_physical_fs(
        fs_root_paths: &[String],
        classifier: &EncounterableResourcePathClassifier,
        nature_aliases: Option<HashMap<String, String>>,
    ) -> ResourcesCollection {
        let physical_fs = vfs::PhysicalFS::new("/");
        let vfs_fs_root = vfs::VfsPath::new(physical_fs);

        let vfs_iter = fs_root_paths
            .iter()
            .flat_map(move |physical_fs_root_path_orig| {
                let physical_fs_root_path: String;
                if let Ok(canonical) = canonicalize(physical_fs_root_path_orig.clone()) {
                    physical_fs_root_path = canonical.to_string_lossy().to_string();
                } else {
                    error!(
                        "Error canonicalizing {}, trying original",
                        physical_fs_root_path_orig
                    );
                    physical_fs_root_path = physical_fs_root_path_orig.to_string();
                }

                let path = vfs_fs_root.join(physical_fs_root_path).unwrap();
                path.walk_dir().unwrap().flatten()
            });

        ResourcesCollection::new(
            vfs_iter.map(EncounterableResource::Vfs).collect(),
            classifier,
            nature_aliases,
        )
    }

    // create a ignore::Walk instance which is a "smart" ignore because it honors .gitigore and .ignore
    // files in the walk path as well as the ignore and other directives passed in via options
    pub fn from_smart_ignore(
        fs_root_paths: &[String],
        classifier: &EncounterableResourcePathClassifier,
        nature_aliases: Option<HashMap<String, String>>,
        ignore_hidden: bool,
    ) -> ResourcesCollection {
        let vfs_iter = fs_root_paths.iter().flat_map(move |root_path| {
            let mut walk_builder = ignore::WalkBuilder::new(root_path);
            walk_builder.hidden(ignore_hidden);
            for cf in &classifier.smart_ignore_conf_files {
                walk_builder.add_custom_ignore_filename(cf);
            }
            walk_builder.build().flatten()
        });

        ResourcesCollection::new(
            vfs_iter.map(EncounterableResource::SmartIgnore).collect(),
            classifier,
            nature_aliases.clone(),
        )
    }

    // create a traditional walkdir::WalkDir which only ignore files based on file names rules passed in
    pub fn from_walk_dir(
        fs_root_paths: &[String],
        classifier: &EncounterableResourcePathClassifier,
        nature_aliases: &Option<HashMap<String, String>>,
    ) -> ResourcesCollection {
        let vfs_iter = fs_root_paths
            .iter()
            .flat_map(move |root_path| walkdir::WalkDir::new(root_path).into_iter().flatten());

        ResourcesCollection::new(
            vfs_iter.map(EncounterableResource::WalkDir).collect(),
            classifier,
            nature_aliases.clone(),
        )
    }

    pub fn from_tasks_lines(
        tasks: &[String],
        classifier: &EncounterableResourcePathClassifier,
        nature_aliases: &Option<HashMap<String, String>>,
    ) -> (Vec<String>, ResourcesCollection) {
        let encounterable: Vec<_> = tasks
            .iter()
            .filter(|line| !line.starts_with('#'))
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_owned())
            .collect();

        (
            encounterable.clone(),
            ResourcesCollection::new(
                encounterable
                    .iter()
                    .map(EncounterableResource::from_deno_task_shell_line)
                    .collect(),
                classifier,
                nature_aliases.clone(),
            ),
        )
    }

    pub fn ignored(&self) -> impl Iterator<Item = EncounteredResource<ContentResource>> + '_ {
        self.encountered()
            .filter(|er| matches!(er, EncounteredResource::Ignored(_, _)))
    }

    pub fn not_ignored(&self) -> impl Iterator<Item = EncounteredResource<ContentResource>> + '_ {
        self.encountered()
            .filter(|er| !matches!(er, EncounteredResource::Ignored(_, _)))
    }

    pub fn capturable_executables(&self) -> impl Iterator<Item = CapturableExecutable> + '_ {
        self.encountered().filter_map(|er| match er {
            EncounteredResource::CapturableExec(_, ce, _) => Some(ce),
            _ => None,
        })
    }

    pub fn encountered(&self) -> impl Iterator<Item = EncounteredResource<ContentResource>> + '_ {
        self.encounterable.iter().map(move |er| {
            let uri = er.uri();
            let mut ero = EncounterableResourceClass {
                nature: None,
                flags: EncounterableResourceFlags::empty(),
            };
            self.classifier.classify(&uri, &mut ero);
            er.encountered(&ero)
        })
    }

    pub fn uniform_resources(
        &self,
    ) -> impl Iterator<Item = anyhow::Result<UniformResource<ContentResource>, Box<dyn Error>>> + '_
    {
        self.encountered()
            .filter_map(move |er: EncounteredResource<ContentResource>| match er {
                EncounteredResource::Resource(resource, _) => {
                    match self.uniform_resource(resource) {
                        Ok(uniform_resource) => Some(Ok(*uniform_resource)),
                        Err(e) => Some(Err(e)), // error will be returned
                    }
                }
                EncounteredResource::CapturableExec(resource, executable, _) => Some(Ok(
                    UniformResource::CapturableExec(CapturableExecResource {
                        resource,
                        executable,
                    }),
                )),
                EncounteredResource::Ignored(_, _)
                | EncounteredResource::NotFile(_, _)
                | EncounteredResource::NotFound(_, _) => None, // these will be filtered via `filter_map`
            })
    }

    pub fn uniform_resource(
        &self,
        cr: ContentResource,
    ) -> Result<Box<UniformResource<ContentResource>>, Box<dyn Error>> {
        // Based on the nature of the resource, we determine the type of UniformResource
        if let Some(cr_nature) = &cr.nature {
            let candidate_nature = if let Some(aliases) = &self.nature_aliases {
                if let Some(alias) = aliases.get(cr_nature.as_str()) {
                    alias.as_str()
                } else {
                    cr_nature.as_str()
                }
            } else {
                cr_nature.as_str()
            };

            match candidate_nature {
                // Match different file extensions
                "html" | "text/html" => {
                    let html = HtmlResource {
                        resource: cr,
                        // TODO parse using
                        //      - https://github.com/y21/tl (performant but not spec compliant)
                        //      - https://github.com/cloudflare/lol-html (more performant, spec compliant)
                        //      - https://github.com/causal-agent/scraper or https://github.com/servo/html5ever directly
                        // create HTML parser presets which can go through all stored HTML, running selectors and putting them into tables?
                    };
                    Ok(Box::new(UniformResource::Html(html)))
                }
                "json" | "jsonc" | "application/json" => {
                    let format = match candidate_nature {
                        "json" | "application/json" => JsonFormat::Json,
                        "jsonc" => JsonFormat::JsonWithComments,
                        _ => JsonFormat::Unknown,
                    };
                    let json = JsonResource {
                        resource: cr,
                        format,
                    };
                    Ok(Box::new(UniformResource::Json(json)))
                }
                "tap" | "toml" | "application/toml" | "yml" | "application/yaml" => {
                    let format = match candidate_nature {
                        "tap" => JsonableTextSchema::TestAnythingProtocol,
                        "toml" | "application/toml" => JsonableTextSchema::Toml,
                        "yml" | "application/yaml" => JsonableTextSchema::Yaml,
                        _ => JsonableTextSchema::Unknown,
                    };
                    let yaml = JsonableTextResource {
                        resource: cr,
                        schema: format,
                    };
                    Ok(Box::new(UniformResource::JsonableText(yaml)))
                }
                "js" | "rs" | "ts" | "puml" => {
                    let interpreter = match candidate_nature {
                        "js" => SourceCodeInterpreter::JavaScript,
                        "puml" => SourceCodeInterpreter::PlantUml,
                        "rs" => SourceCodeInterpreter::Rust,
                        "ts" => SourceCodeInterpreter::TypeScript,
                        _ => SourceCodeInterpreter::Unknown,
                    };
                    let source_code = SourceCodeResource {
                        resource: cr,
                        interpreter,
                    };
                    Ok(Box::new(UniformResource::SourceCode(source_code)))
                }
                "md" | "mdx" | "text/markdown" => {
                    let markdown = MarkdownResource { resource: cr };
                    Ok(Box::new(UniformResource::Markdown(markdown)))
                }
                "txt" | "text/plain" => {
                    let plain_text = PlainTextResource { resource: cr };
                    Ok(Box::new(UniformResource::PlainText(plain_text)))
                }
                "png" | "gif" | "tiff" | "jpg" | "jpeg" => {
                    // TODO: need to implement `infer` crate auto-detection
                    let image = ImageResource { resource: cr };
                    Ok(Box::new(UniformResource::Image(image)))
                }
                "svg" | "image/svg+xml" | "xml" | "text/xml" | "application/xml" => {
                    let schema = match candidate_nature {
                        "svg" | "image/svg+xml" => XmlSchema::Svg,
                        "xml" | "text/xml" | "application/xml" => XmlSchema::Unknown,
                        _ => XmlSchema::Unknown,
                    };
                    let xml = XmlResource {
                        resource: cr,
                        schema,
                    };
                    Ok(Box::new(UniformResource::Xml(xml)))
                }
                _ => Ok(Box::new(UniformResource::Unknown(cr, None))),
            }
        } else {
            Err(format!(
                "Unable to obtain nature for {} from supplied resource",
                cr.uri
            )
            .into())
        }
    }
}

/// Extracts various path-related information from the given root path and entry.
///
/// # Parameters
///
/// * `root_path` - The root directory path as a reference to a `Path`.
/// * `root_path_entry` - The file or directory entry path as a reference to a `Path`.
///
/// # Returns
///
/// A tuple containing:
/// - `file_path_abs`: Absolute path of `root_path_entry`.
/// - `file_path_rel_parent`: The parent directory of `root_path_entry`.
/// - `file_path_rel`: Path of `root_path_entry` relative to `root_path`.
/// - `file_basename`: The basename of `root_path_entry` (with extension).
/// - `file_extn`: The file extension of `root_path_entry` (without `.`).
///
/// # Errors
///
/// Returns `None` if any of the path conversions fail.
pub fn extract_path_info(
    root_path: &Path,
    root_path_entry: &Path,
) -> Option<(PathBuf, PathBuf, PathBuf, String, Option<String>)> {
    let file_path_abs = root_path_entry.canonicalize().ok()?;
    let file_path_rel_parent = root_path_entry.parent()?.to_path_buf();
    let file_path_rel = root_path_entry.strip_prefix(root_path).ok()?.to_path_buf();
    let file_basename = root_path_entry.file_name()?.to_str()?.to_string();
    let file_extn = root_path_entry
        .extension()
        .and_then(|s| s.to_str())
        .map(String::from);

    Some((
        file_path_abs,
        file_path_rel_parent,
        file_path_rel,
        file_basename,
        file_extn,
    ))
}
