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
use regex::RegexSet;
use serde_json::Value as JsonValue;
use sha1::{Digest, Sha1};

use crate::shell::*;

use crate::frontmatter::frontmatter;

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
pub type JsonValueSupplier = Box<dyn Fn() -> Result<Box<JsonValue>, Box<dyn Error>>>;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EncounterableResourceFlags: u32 {
        const CONTENT_ACQUIRABLE       = 0b00000001;
        const IGNORE_BY_NAME_REQUESTED = 0b00000010;

        // there might be different types of "ignore" flags, create a union of
        // all ignores into one when you don't care which one is set.
        const IGNORABLE                = ContentResourceFlags::IGNORE_BY_NAME_REQUESTED.bits();

        // TODO: see https://docs.rs/bitflags/latest/bitflags/#externally-defined-flags
        // const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EncounteredResourceFlags: u32 {
        const CONTENT_ACQUIRABLE       = EncounterableResourceFlags::CONTENT_ACQUIRABLE.bits();
        const IGNORE_BY_NAME_REQUESTED = EncounterableResourceFlags::IGNORE_BY_NAME_REQUESTED.bits();

        const IS_FILE                  = EncounteredResourceFlags::IGNORE_BY_NAME_REQUESTED.bits() << 1;
        const IS_DIRECTORY             = EncounteredResourceFlags::IS_FILE.bits() << 1;
        const IS_SYMLINK               = EncounteredResourceFlags::IS_DIRECTORY.bits() << 1;

        // there might be different types of "ignore" flags, create a union of
        // all ignores into one when you don't care which one is set.
        const IGNORABLE                = EncounteredResourceFlags::IGNORE_BY_NAME_REQUESTED.bits();

        // TODO: see https://docs.rs/bitflags/latest/bitflags/#externally-defined-flags
        // const _ = !0;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ContentResourceFlags: u32 {
        const CONTENT_ACQUIRABLE       = EncounteredResourceFlags::CONTENT_ACQUIRABLE.bits();
        const IGNORE_BY_NAME_REQUESTED = EncounteredResourceFlags::IGNORE_BY_NAME_REQUESTED.bits();

        // there might be different types of "ignore" flags, create a union of
        // all ignores into one when you don't care which one is set.
        const IGNORABLE                = ContentResourceFlags::IGNORE_BY_NAME_REQUESTED.bits();

        // TODO: see https://docs.rs/bitflags/latest/bitflags/#externally-defined-flags
        // const _ = !0;
    }
}
// pub is_ignored: bool,
// pub acquire_content: bool,

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

pub struct HtmlResource<Resource> {
    pub resource: Resource,
    pub head_meta: HashMap<String, String>,
}

pub struct ImageResource<Resource> {
    pub resource: Resource,
    pub image_meta: HashMap<String, String>,
}

pub struct JsonResource<Resource> {
    pub resource: Resource,
    pub content: Option<JsonValueSupplier>, // The actual JSON content
}

pub struct MarkdownResource<Resource> {
    pub resource: Resource,
}

pub struct PlainTextResource<Resource> {
    pub resource: Resource,
}

pub struct SoftwarePackageDxResource<Resource> {
    pub resource: Resource,
}

pub struct SvgResource<Resource> {
    pub resource: Resource,
}

pub struct TestAnythingResource<Resource> {
    pub resource: Resource,
}
pub struct TomlResource<Resource> {
    pub resource: Resource,
    pub content: Option<JsonValueSupplier>, // transformed to JSON content
}

pub struct YamlResource<Resource> {
    pub resource: Resource,
    pub content: Option<JsonValueSupplier>, // transformed to JSON content
}

pub enum UniformResource<Resource> {
    CapturableExec(CapturableExecResource<Resource>),
    Html(HtmlResource<Resource>),
    Image(ImageResource<Resource>),
    Json(JsonResource<Resource>),
    Markdown(MarkdownResource<Resource>),
    PlainText(PlainTextResource<Resource>),
    SpdxJson(SoftwarePackageDxResource<Resource>), // TODO: SPDX comes in 5 flavors (see https://spdx.dev/learn/overview/)
    Svg(SvgResource<Resource>),
    Tap(TestAnythingResource<Resource>),
    Toml(TomlResource<Resource>),
    Yaml(YamlResource<Resource>),
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
            UniformResource::Markdown(md) => &md.resource.uri,
            UniformResource::PlainText(txt) => &txt.resource.uri,
            UniformResource::SpdxJson(spdx) => &spdx.resource.uri,
            UniformResource::Svg(svg) => &svg.resource.uri,
            UniformResource::Tap(tap) => &tap.resource.uri,
            UniformResource::Toml(toml) => &toml.resource.uri,
            UniformResource::Yaml(yaml) => &yaml.resource.uri,
            UniformResource::Unknown(cr, _alternate) => &cr.uri,
        }
    }

    fn nature(&self) -> &Option<String> {
        match self {
            crate::resource::UniformResource::CapturableExec(cer) => &cer.resource.nature,
            crate::resource::UniformResource::Html(html) => &html.resource.nature,
            crate::resource::UniformResource::Image(img) => &img.resource.nature,
            crate::resource::UniformResource::Json(json) => &json.resource.nature,
            crate::resource::UniformResource::Markdown(md) => &md.resource.nature,
            crate::resource::UniformResource::PlainText(txt) => &txt.resource.nature,
            crate::resource::UniformResource::SpdxJson(spdx) => &spdx.resource.nature,
            crate::resource::UniformResource::Svg(svg) => &svg.resource.nature,
            crate::resource::UniformResource::Tap(tap) => &tap.resource.nature,
            crate::resource::UniformResource::Toml(toml) => &toml.resource.nature,
            crate::resource::UniformResource::Yaml(yaml) => &yaml.resource.nature,
            crate::resource::UniformResource::Unknown(_cr, _alternate) => &None::<String>,
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

pub struct EncounterableResourceOptions {
    pub flags: EncounterableResourceFlags,
}

#[derive(Debug)]
pub struct EncounteredResourceMetaData {
    pub flags: EncounteredResourceFlags,
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

        Ok(EncounteredResourceMetaData {
            flags,
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

        Ok(EncounteredResourceMetaData {
            flags,
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
        options: &EncounterableResourceOptions,
    ) -> EncounteredResourceContentSuppliers {
        let binary: Option<BinaryContentSupplier>;
        let text: Option<TextContentSupplier>;

        if options
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
        options: &EncounterableResourceOptions,
    ) -> EncounteredResourceContentSuppliers {
        let binary: Option<BinaryContentSupplier>;
        let text: Option<TextContentSupplier>;

        if options
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
}

pub enum EncounteredResource<T> {
    Ignored(String),
    NotFound(String),
    NotFile(String),
    Resource(T),
    CapturableExec(T, CapturableExecutable),
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
        }
    }

    pub fn _path(&self) -> Option<&Path> {
        match self {
            EncounterableResource::WalkDir(de) => Some(de.path()),
            EncounterableResource::SmartIgnore(de) => Some(de.path()),
            EncounterableResource::Vfs(_path) => None,
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
        }
    }

    pub fn content_suppliers(
        &self,
        options: &EncounterableResourceOptions,
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
        }
    }

    pub fn capturable_executable(
        &self,
        ce_rules: &CapturableExecutableRegexRules,
    ) -> Option<CapturableExecutable> {
        match self {
            EncounterableResource::WalkDir(de) => ce_rules.path_capturable_executable(de.path()),
            EncounterableResource::SmartIgnore(de) => {
                ce_rules.path_capturable_executable(de.path())
            }
            EncounterableResource::Vfs(path) => ce_rules.uri_capturable_executable(path.as_str()),
        }
    }

    pub fn encountered(
        &self,
        options: &EncounterableResourceOptions,
    ) -> EncounteredResource<ContentResource> {
        let uri = self.uri();
        if options
            .flags
            .contains(EncounterableResourceFlags::IGNORE_BY_NAME_REQUESTED)
        {
            return EncounteredResource::Ignored(uri);
        }

        let metadata = match self.meta_data() {
            Ok(metadata) => {
                // TODO: what about symlinks?
                if !metadata.flags.contains(EncounteredResourceFlags::IS_FILE) {
                    return EncounteredResource::NotFile(uri);
                }
                metadata
            }
            Err(_) => return EncounteredResource::NotFound(uri),
        };

        // typically the nature is a the file's extension
        let nature = uri.rsplit_once('.').map(|(_, ext)| ext.to_string());
        let content_suppliers = self.content_suppliers(options);

        EncounteredResource::Resource(ContentResource {
            flags: ContentResourceFlags::from_bits_truncate(options.flags.bits()),
            uri: uri.to_string(),
            nature,
            size: Some(metadata.file_size),
            created_at: metadata.created_at,
            last_modified_at: metadata.last_modified_at,
            content_binary_supplier: content_suppliers.binary,
            content_text_supplier: content_suppliers.text,
        })
    }
}

pub enum CapturableExecutable {
    UriShellExecutive(Box<dyn ShellExecutive>, String, String, bool),
    RequestedButNoNature(String, regex::Regex),
    RequestedButNotExecutable(String),
}

impl CapturableExecutable {
    pub fn uri(&self) -> &str {
        match self {
            CapturableExecutable::UriShellExecutive(_, uri, _, _)
            | CapturableExecutable::RequestedButNoNature(uri, _)
            | CapturableExecutable::RequestedButNotExecutable(uri) => uri.as_str(),
        }
    }

    pub fn executed_result_as_text(
        &self,
        std_in: ShellStdIn,
    ) -> anyhow::Result<(String, String, bool), serde_json::Value> {
        match self {
            CapturableExecutable::UriShellExecutive(executive, _, nature, is_batched_sql) => {
                match executive.execute(std_in) {
                    Ok(shell_result) => {
                        if shell_result.success() {
                            Ok((shell_result.stdout, nature.clone(), *is_batched_sql))
                        } else {
                            Err(serde_json::json!({
                                "src": self.uri(),
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
                        "issue": "[CapturableExecutable::TextFromExecutableUri.executed_text] execution error",
                        "rust-err": format!("{:?}", err),
                        "nature": nature,
                    })),
                }
            }
            CapturableExecutable::RequestedButNoNature(src, regex) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNoNature.executed_sql] unable to determine nature",
                "remediation": "make sure that the regular expression has a `nature` named capture group",
                "regex": format!("{:?}", regex),
            })),
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
            CapturableExecutable::UriShellExecutive(executive, _, nature, is_batched_sql) => {
                match executive.execute(std_in) {
                    Ok(shell_result) => {
                        if shell_result.success() {
                            let captured_text = shell_result.stdout;
                            let value: serde_json::Result<serde_json::Value> =
                                serde_json::from_str(&captured_text);
                            match value {
                                Ok(value) => Ok((value, nature.clone(), *is_batched_sql)),
                                Err(_) => Err(serde_json::json!({
                                    "src": self.uri(),
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
                }
            }
            CapturableExecutable::RequestedButNoNature(src, regex) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNoNature.executed_result_as_json] unable to determine nature",
                "remediation": "make sure that the regular expression has a `nature` named capture group",
                "regex": format!("{:?}", regex),
            })),
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
            CapturableExecutable::UriShellExecutive(executive, _, nature, is_batched_sql) => {
                if *is_batched_sql {
                    match executive.execute(std_in) {
                        Ok(shell_result) => {
                            if shell_result.status.success() {
                                Ok((shell_result.stdout, nature.clone()))
                            } else {
                                Err(serde_json::json!({
                                    "src": self.uri(),
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
                            "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_sql] execution error",
                            "rust-err": format!("{:?}", err),
                            "nature": nature,
                        })),
                    }
                } else {
                    Err(serde_json::json!({
                        "src": self.uri(),
                        "issue": "[CapturableExecutable::TextFromExecutableUri.executed_result_as_sql] is not classified as batch SQL",
                        "nature": nature,
                    }))
                }
            }
            CapturableExecutable::RequestedButNoNature(src, regex) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNoNature.executed_result_as_sql] unable to determine nature",
                "remediation": "make sure that the regular expression has a `nature` named capture group",
                "regex": format!("{:?}", regex),
            })),
            CapturableExecutable::RequestedButNotExecutable(src) => Err(serde_json::json!({
                "src": src,
                "issue": "[CapturableExecutable::RequestedButNotExecutable.executed_result_as_sql] executable permissions not set",
                "remediation": "make sure that script has executable permissions set",
            })),
        }
    }
}

const DEFAULT_CAPTURE_EXEC_REGEX_PATTERN: &str = r"surveilr\[(?P<nature>[^\]]*)\]";
const DEFAULT_CAPTURE_SQL_EXEC_REGEX_PATTERN: &str = r"surveilr-SQL";

pub trait CapturableExecutableSupplier {
    fn shell_executive(&self) -> Box<dyn ShellExecutive>;
}

#[derive(Debug, Clone)]
pub struct CapturableExecutableRegexRules {
    pub capturable_regexs: Vec<regex::Regex>,
    pub capturable_sql_set: RegexSet,
}

impl CapturableExecutableRegexRules {
    pub fn new(
        capturable_executables_regexs: Option<&[regex::Regex]>,
        captured_exec_sql_regexs: Option<&[regex::Regex]>,
    ) -> anyhow::Result<Self> {
        // Constructor can fail due to RegexSet::new
        let is_capturable = match capturable_executables_regexs {
            Some(capturable_executables_regexs) => capturable_executables_regexs.to_vec(),
            None => vec![regex::Regex::new(DEFAULT_CAPTURE_EXEC_REGEX_PATTERN)?],
        };
        let is_capturable_sql = match captured_exec_sql_regexs {
            Some(captured_exec_sql_regexs) => {
                RegexSet::new(captured_exec_sql_regexs.iter().map(|r| r.as_str()))?
            }
            None => RegexSet::new([DEFAULT_CAPTURE_SQL_EXEC_REGEX_PATTERN])?,
        };

        Ok(CapturableExecutableRegexRules {
            capturable_regexs: is_capturable,
            capturable_sql_set: is_capturable_sql,
        })
    }

    // check if URI is executable based only on the filename pattern
    pub fn uri_capturable_executable(&self, uri: &str) -> Option<CapturableExecutable> {
        let mut ce: Option<CapturableExecutable> = None;

        let executable_file_uri = uri.to_string();
        if self.capturable_sql_set.is_match(uri) {
            ce = Some(CapturableExecutable::UriShellExecutive(
                Box::new(executable_file_uri.clone()), // String has the `ShellExecutive` trait
                executable_file_uri,
                String::from("surveilr-SQL"),
                true,
            ));
        } else {
            for re in self.capturable_regexs.iter() {
                if let Some(caps) = re.captures(uri) {
                    if let Some(nature) = caps.name("nature") {
                        ce = Some(CapturableExecutable::UriShellExecutive(
                            Box::new(executable_file_uri.clone()), // String has the `ShellExecutive` trait
                            executable_file_uri,
                            String::from(nature.as_str()),
                            false,
                        ));
                        break;
                    } else {
                        ce = Some(CapturableExecutable::RequestedButNoNature(
                            executable_file_uri,
                            re.clone(),
                        ));
                        break;
                    }
                }
            }
        }
        ce
    }

    // check if URI is executable based the filename pattern first, then physical FS validation of execute permission
    pub fn path_capturable_executable(
        &self,
        path: &std::path::Path,
    ) -> Option<CapturableExecutable> {
        let uri_ce = self.uri_capturable_executable(path.to_str().unwrap());
        if uri_ce.is_some() {
            if path.is_executable() {
                return uri_ce;
            } else {
                return Some(CapturableExecutable::RequestedButNotExecutable(
                    path.to_string_lossy().to_string(),
                ));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct ResourcesCollectionOptions {
    pub ignore_paths_regexs: Vec<regex::Regex>,
    pub ingest_content_regexs: Vec<regex::Regex>,
    pub capturable_executables_regexs: Vec<regex::Regex>,
    pub captured_exec_sql_regexs: Vec<regex::Regex>,
    pub nature_bind: HashMap<String, String>,
}

pub struct ResourcesCollection {
    pub encounterable: Vec<EncounterableResource>,
    pub ignore_paths_regex_set: RegexSet,
    pub ingest_content_regex_set: RegexSet,
    pub ce_rules: CapturableExecutableRegexRules,
    pub nature_bind: HashMap<String, String>,
}

impl ResourcesCollection {
    pub fn new(
        encounterable: Vec<EncounterableResource>,
        options: &ResourcesCollectionOptions,
    ) -> ResourcesCollection {
        let ignore_paths =
            RegexSet::new(options.ignore_paths_regexs.iter().map(|r| r.as_str())).unwrap();
        let acquire_content =
            RegexSet::new(options.ingest_content_regexs.iter().map(|r| r.as_str())).unwrap();
        let ce_rules = CapturableExecutableRegexRules::new(
            Some(&options.capturable_executables_regexs),
            Some(&options.captured_exec_sql_regexs),
        )
        .unwrap();

        ResourcesCollection {
            encounterable,
            ignore_paths_regex_set: ignore_paths,
            ingest_content_regex_set: acquire_content,
            ce_rules,
            nature_bind: options.nature_bind.clone(),
        }
    }

    // create a physical file system mapped via VFS, mainly for testing and experimental use
    pub fn from_vfs_physical_fs(
        fs_root_paths: &[String],
        options: &ResourcesCollectionOptions,
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
                    eprintln!(
                        "Error canonicalizing {}, trying original",
                        physical_fs_root_path_orig
                    );
                    physical_fs_root_path = physical_fs_root_path_orig.to_string();
                }

                let path = vfs_fs_root.join(physical_fs_root_path).unwrap();
                path.walk_dir().unwrap().flatten()
            });

        ResourcesCollection::new(vfs_iter.map(EncounterableResource::Vfs).collect(), options)
    }

    // create a ignore::Walk instance which is a "smart" ignore because it honors .gitigore and .ignore
    // files in the walk path as well as the ignore and other directives passed in via options
    pub fn from_smart_ignore(
        fs_root_paths: &[String],
        options: &ResourcesCollectionOptions,
        include_hidden: bool,
    ) -> ResourcesCollection {
        let vfs_iter = fs_root_paths.iter().flat_map(move |root_path| {
            let ignorable_walk = if include_hidden {
                ignore::WalkBuilder::new(root_path).hidden(false).build()
            } else {
                ignore::Walk::new(root_path)
            };
            ignorable_walk.into_iter().flatten()
        });

        ResourcesCollection::new(
            vfs_iter.map(EncounterableResource::SmartIgnore).collect(),
            options,
        )
    }

    // create a traditional walkdir::WalkDir which only ignore files based on file names rules passed in
    pub fn from_walk_dir(
        fs_root_paths: &[String],
        options: &ResourcesCollectionOptions,
    ) -> ResourcesCollection {
        let vfs_iter = fs_root_paths
            .iter()
            .flat_map(move |root_path| walkdir::WalkDir::new(root_path).into_iter().flatten());

        ResourcesCollection::new(
            vfs_iter.map(EncounterableResource::WalkDir).collect(),
            options,
        )
    }

    pub fn ignored(&self) -> impl Iterator<Item = &EncounterableResource> + '_ {
        self.encounterable
            .iter()
            .filter(|er| self.ignore_paths_regex_set.is_match(&er.uri()))
    }

    pub fn not_ignored(&self) -> impl Iterator<Item = &EncounterableResource> + '_ {
        self.encounterable
            .iter()
            .filter(|er| !self.ignore_paths_regex_set.is_match(&er.uri()))
    }

    pub fn encountered(&self) -> impl Iterator<Item = EncounteredResource<ContentResource>> + '_ {
        self.encounterable.iter().map(move |er| {
            let uri = er.uri();
            let mut flags = EncounterableResourceFlags::empty();
            if self.ignore_paths_regex_set.is_match(&uri) {
                flags.insert(EncounterableResourceFlags::IGNORE_BY_NAME_REQUESTED);
            }
            if self.ingest_content_regex_set.is_match(&uri) {
                flags.insert(EncounterableResourceFlags::CONTENT_ACQUIRABLE);
            }
            let ero = EncounterableResourceOptions { flags };
            let initial_guess = er.encountered(&ero);

            match &initial_guess {
                EncounteredResource::Resource(cr) => {
                    if let Some(executable) = match er {
                        EncounterableResource::WalkDir(de) => {
                            // this strictly checks that path is executable
                            self.ce_rules.path_capturable_executable(de.path())
                        }
                        EncounterableResource::SmartIgnore(de) => {
                            // this strictly checks that path is executable
                            self.ce_rules.path_capturable_executable(de.path())
                        }
                        EncounterableResource::Vfs(path) => {
                            // this only checks that naming rules are satisfied
                            self.ce_rules.uri_capturable_executable(path.as_str())
                        }
                    } {
                        EncounteredResource::CapturableExec(
                            ContentResource {
                                flags: cr.flags,
                                uri: cr.uri.clone(),
                                nature: cr.nature.clone(),
                                size: cr.size,
                                created_at: cr.created_at,
                                last_modified_at: cr.last_modified_at,
                                content_binary_supplier: None, // TODO: figure out how to clone this
                                content_text_supplier: None,   // TODO: figure out how to clone this
                            },
                            executable,
                        )
                    } else {
                        initial_guess
                    }
                }
                _ => initial_guess,
            }
        })
    }

    pub fn capturable_executables(&self) -> impl Iterator<Item = CapturableExecutable> + '_ {
        self.encounterable
            .iter()
            // "smart" means to try the path name and ensure that file is executable on disk
            .filter_map(|rwe| rwe.capturable_executable(&self.ce_rules))
    }

    pub fn uniform_resources(
        &self,
    ) -> impl Iterator<Item = anyhow::Result<UniformResource<ContentResource>, Box<dyn Error>>> + '_
    {
        self.encountered()
            .filter_map(move |er: EncounteredResource<ContentResource>| match er {
                EncounteredResource::Resource(resource) => {
                    match self.uniform_resource(resource) {
                        Ok(uniform_resource) => Some(Ok(*uniform_resource)),
                        Err(e) => Some(Err(e)), // error will be returned
                    }
                }
                EncounteredResource::CapturableExec(resource, executable) => Some(Ok(
                    UniformResource::CapturableExec(CapturableExecResource {
                        resource,
                        executable,
                    }),
                )),
                EncounteredResource::Ignored(_)
                | EncounteredResource::NotFile(_)
                | EncounteredResource::NotFound(_) => None, // these will be filtered via `filter_map`
            })
    }

    pub fn uniform_resource(
        &self,
        cr: ContentResource,
    ) -> Result<Box<UniformResource<ContentResource>>, Box<dyn Error>> {
        // Based on the nature of the resource, we determine the type of UniformResource
        if let Some(supplied_nature) = &cr.nature {
            let mut candidate_nature = supplied_nature.as_str();
            let try_alternate_nature = self.nature_bind.get(candidate_nature);
            if let Some(alternate_bind) = try_alternate_nature {
                candidate_nature = alternate_bind
            }

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
                        head_meta: HashMap::new(),
                    };
                    Ok(Box::new(UniformResource::Html(html)))
                }
                "json" | "jsonc" | "application/json" => {
                    if cr.uri.ends_with(".spdx.json") {
                        let spdx_json = SoftwarePackageDxResource { resource: cr };
                        Ok(Box::new(UniformResource::SpdxJson(spdx_json)))
                    } else {
                        let json = JsonResource {
                            resource: cr,
                            content: None, // TODO parse using serde
                        };
                        Ok(Box::new(UniformResource::Json(json)))
                    }
                }
                "yml" | "application/yaml" => {
                    let yaml = YamlResource {
                        resource: cr,
                        content: None, // TODO parse using serde
                    };
                    Ok(Box::new(UniformResource::Yaml(yaml)))
                }
                "toml" | "application/toml" => {
                    let toml = TomlResource {
                        resource: cr,
                        content: None, // TODO parse using serde
                    };
                    Ok(Box::new(UniformResource::Toml(toml)))
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
                    let image = ImageResource {
                        resource: cr,
                        image_meta: HashMap::new(), // TODO add meta data, infer type from content
                    };
                    Ok(Box::new(UniformResource::Image(image)))
                }
                "svg" | "image/svg+xml" => {
                    let svg = SvgResource { resource: cr };
                    Ok(Box::new(UniformResource::Svg(svg)))
                }
                "tap" => {
                    let tap = TestAnythingResource { resource: cr };
                    Ok(Box::new(UniformResource::Tap(tap)))
                }
                _ => Ok(Box::new(UniformResource::Unknown(
                    cr,
                    try_alternate_nature.cloned(),
                ))),
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
