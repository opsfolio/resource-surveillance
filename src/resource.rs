use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::canonicalize;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use regex::RegexSet;
use serde_json::Value as JsonValue;
use sha1::{Digest, Sha1};

use crate::capturable::*;
use crate::subprocess::*;

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

pub struct ContentResource {
    pub uri: String,
    pub nature: Option<String>,
    pub size: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub capturable_executable: Option<CapturableExecutable>,
    pub content_binary_supplier: Option<BinaryContentSupplier>,
    pub content_text_supplier: Option<TextContentSupplier>,
    pub capturable_exec_binary_supplier: Option<BinaryExecOutputSupplier>,
    pub capturable_exec_text_supplier: Option<TextExecOutputSupplier>,
}

pub struct CapturableExecResource<Resource> {
    pub executable: Resource,
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
            UniformResource::CapturableExec(cer) => &cer.executable.uri,
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
            crate::resource::UniformResource::CapturableExec(cer) => &cer.executable.nature,
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

pub struct UniformResourceBuilder {
    pub nature_bind: HashMap<String, String>,
}

impl UniformResourceSupplier<ContentResource> for UniformResourceBuilder {
    fn uniform_resource(
        &self,
        resource: ContentResource,
    ) -> Result<Box<UniformResource<ContentResource>>, Box<dyn Error>> {
        if resource.capturable_executable.is_some() {
            return Ok(Box::new(UniformResource::CapturableExec(
                CapturableExecResource {
                    executable: resource,
                },
            )));
        }

        // Based on the nature of the resource, we determine the type of UniformResource
        if let Some(supplied_nature) = &resource.nature {
            let mut candidate_nature = supplied_nature.as_str();
            let try_alternate_nature = self.nature_bind.get(candidate_nature);
            if let Some(alternate_bind) = try_alternate_nature {
                candidate_nature = alternate_bind
            }

            match candidate_nature {
                // Match different file extensions
                "html" | "text/html" => {
                    let html = HtmlResource {
                        resource,
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
                    if resource.uri.ends_with(".spdx.json") {
                        let spdx_json = SoftwarePackageDxResource { resource };
                        Ok(Box::new(UniformResource::SpdxJson(spdx_json)))
                    } else {
                        let json = JsonResource {
                            resource,
                            content: None, // TODO parse using serde
                        };
                        Ok(Box::new(UniformResource::Json(json)))
                    }
                }
                "yml" | "application/yaml" => {
                    let yaml = YamlResource {
                        resource,
                        content: None, // TODO parse using serde
                    };
                    Ok(Box::new(UniformResource::Yaml(yaml)))
                }
                "toml" | "application/toml" => {
                    let toml = TomlResource {
                        resource,
                        content: None, // TODO parse using serde
                    };
                    Ok(Box::new(UniformResource::Toml(toml)))
                }
                "md" | "mdx" | "text/markdown" => {
                    let markdown = MarkdownResource { resource };
                    Ok(Box::new(UniformResource::Markdown(markdown)))
                }
                "txt" | "text/plain" => {
                    let plain_text = PlainTextResource { resource };
                    Ok(Box::new(UniformResource::PlainText(plain_text)))
                }
                "png" | "gif" | "tiff" | "jpg" | "jpeg" => {
                    let image = ImageResource {
                        resource,
                        image_meta: HashMap::new(), // TODO add meta data, infer type from content
                    };
                    Ok(Box::new(UniformResource::Image(image)))
                }
                "svg" | "image/svg+xml" => {
                    let svg = SvgResource { resource };
                    Ok(Box::new(UniformResource::Svg(svg)))
                }
                "tap" => {
                    let tap = TestAnythingResource { resource };
                    Ok(Box::new(UniformResource::Tap(tap)))
                }
                _ => Ok(Box::new(UniformResource::Unknown(
                    resource,
                    try_alternate_nature.cloned(),
                ))),
            }
        } else {
            Err(format!(
                "Unable to obtain nature for {} from supplied resource",
                resource.uri
            )
            .into())
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
pub struct EncounterableResourceOptions {
    pub is_ignored: bool,
    pub acquire_content: bool,
    pub capturable_executable: Option<CapturableExecutable>,
}

#[derive(Debug)]
pub struct EncounterableResourceMetaData {
    pub is_file: bool,
    pub is_dir: bool,
    pub file_size: u64,
    pub created_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>,
    pub last_modified_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>,
}

impl EncounterableResourceMetaData {
    pub fn from_fs_path(fs_path: &Path) -> anyhow::Result<EncounterableResourceMetaData> {
        let is_file: bool;
        let is_dir: bool;
        let file_size: u64;
        let created_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>;
        let last_modified_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>;

        match fs::metadata(fs_path) {
            Ok(metadata) => {
                is_file = metadata.is_file();
                is_dir = metadata.is_dir();
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

        Ok(EncounterableResourceMetaData {
            is_file,
            is_dir,
            file_size,
            created_at,
            last_modified_at,
        })
    }

    pub fn from_vfs_path(vfs_path: &vfs::VfsPath) -> anyhow::Result<EncounterableResourceMetaData> {
        let is_file: bool;
        let is_dir: bool;

        let metadata = match vfs_path.metadata() {
            Ok(metadata) => {
                match metadata.file_type {
                    vfs::VfsFileType::File => {
                        is_file = true;
                        is_dir = false;
                    }
                    vfs::VfsFileType::Directory => {
                        is_file = false;
                        is_dir = true;
                    }
                };
                metadata
            }
            Err(err) => {
                let context = format!("ResourceContentMetaData::from_vfs_path({:?})", vfs_path);
                return Err(anyhow::Error::new(err).context(context));
            }
        };

        Ok(EncounterableResourceMetaData {
            is_file,
            is_dir,
            file_size: metadata.len,
            created_at: None,
            last_modified_at: None,
        })
    }
}

pub struct ResourceContentSuppliers {
    pub text: Option<TextContentSupplier>,
    pub binary: Option<BinaryContentSupplier>,
}

impl ResourceContentSuppliers {
    pub fn from_fs_path(
        fs_path: &Path,
        options: &EncounterableResourceOptions,
    ) -> ResourceContentSuppliers {
        let binary: Option<BinaryContentSupplier>;
        let text: Option<TextContentSupplier>;

        if options.acquire_content {
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

        ResourceContentSuppliers { binary, text }
    }

    pub fn from_vfs_path(
        vfs_path: &vfs::VfsPath,
        options: &EncounterableResourceOptions,
    ) -> ResourceContentSuppliers {
        let binary: Option<BinaryContentSupplier>;
        let text: Option<TextContentSupplier>;

        if options.acquire_content {
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

        ResourceContentSuppliers { text, binary }
    }
}

pub enum EncounterableResource {
    WalkDir(walkdir::DirEntry),
    SmartIgnore(ignore::DirEntry),
    Vfs(vfs::VfsPath),
}

#[derive(Debug)]
pub enum EncounteredResource<T> {
    Ignored(String),
    NotFound(String),
    NotFile(String),
    Resource(T),
}

impl EncounterableResource {
    fn uri(&self) -> String {
        match self {
            EncounterableResource::WalkDir(de) => de.path().to_string_lossy().to_string(),
            EncounterableResource::SmartIgnore(de) => de.path().to_string_lossy().to_string(),
            EncounterableResource::Vfs(path) => path.as_str().to_string(),
        }
    }

    fn _path(&self) -> Option<&Path> {
        match self {
            EncounterableResource::WalkDir(de) => Some(de.path()),
            EncounterableResource::SmartIgnore(de) => Some(de.path()),
            EncounterableResource::Vfs(_path) => None,
        }
    }

    pub fn meta_data(&self) -> anyhow::Result<EncounterableResourceMetaData> {
        match self {
            EncounterableResource::WalkDir(de) => {
                EncounterableResourceMetaData::from_fs_path(de.path())
            }
            EncounterableResource::SmartIgnore(de) => {
                EncounterableResourceMetaData::from_fs_path(de.path())
            }
            EncounterableResource::Vfs(path) => EncounterableResourceMetaData::from_vfs_path(path),
        }
    }

    pub fn content_suppliers(
        &self,
        options: &EncounterableResourceOptions,
    ) -> ResourceContentSuppliers {
        match self {
            EncounterableResource::WalkDir(de) => {
                ResourceContentSuppliers::from_fs_path(de.path(), options)
            }
            EncounterableResource::SmartIgnore(de) => {
                ResourceContentSuppliers::from_fs_path(de.path(), options)
            }
            EncounterableResource::Vfs(path) => {
                ResourceContentSuppliers::from_vfs_path(path, options)
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

    pub fn encountered_resource(
        &self,
        options: &EncounterableResourceOptions,
    ) -> EncounteredResource<ContentResource> {
        let uri = self.uri();
        if options.is_ignored {
            return EncounteredResource::Ignored(uri);
        }

        let metadata = match self.meta_data() {
            Ok(metadata) => {
                if !metadata.is_file {
                    return EncounteredResource::NotFile(uri);
                }
                metadata
            }
            Err(_) => return EncounteredResource::NotFound(uri),
        };

        // typically the nature is a the file's extension
        let nature = uri.rsplit_once('.').map(|(_, ext)| ext.to_string());

        let capturable_executable: Option<CapturableExecutable>;
        let capturable_exec_binary_supplier: Option<BinaryExecOutputSupplier>;
        let capturable_exec_text_supplier: Option<TextExecOutputSupplier>;

        if let Some(capturable) = &options.capturable_executable {
            capturable_executable = Some(capturable.clone());
            capturable_exec_binary_supplier = capturable.executable_content_binary();
            capturable_exec_text_supplier = capturable.executable_content_text();
        } else {
            capturable_executable = None;
            capturable_exec_binary_supplier = None;
            capturable_exec_text_supplier = None;
        }

        let content_suppliers = self.content_suppliers(options);

        EncounteredResource::Resource(ContentResource {
            uri: uri.to_string(),
            nature,
            size: Some(metadata.file_size),
            created_at: metadata.created_at,
            last_modified_at: metadata.last_modified_at,
            capturable_executable,
            content_binary_supplier: content_suppliers.binary,
            content_text_supplier: content_suppliers.text,
            capturable_exec_binary_supplier,
            capturable_exec_text_supplier,
        })
    }
}

#[derive(Debug)]

pub struct ResourceCollectionOptions {
    pub ignore_paths_regexs: Vec<regex::Regex>,
    pub ingest_content_regexs: Vec<regex::Regex>,
    pub capturable_executables_regexs: Vec<regex::Regex>,
    pub captured_exec_sql_regexs: Vec<regex::Regex>,
    pub nature_bind: HashMap<String, String>,
}

pub struct ResourceCollection {
    pub walked: Vec<EncounterableResource>,
    pub ignore_paths_regex_set: RegexSet,
    pub ingest_content_regex_set: RegexSet,
    pub ce_rules: CapturableExecutableRegexRules,
    pub ur_builder: UniformResourceBuilder,
}

impl ResourceCollection {
    pub fn new(
        walked: Vec<EncounterableResource>,
        options: &ResourceCollectionOptions,
    ) -> ResourceCollection {
        let ignore_paths =
            RegexSet::new(options.ignore_paths_regexs.iter().map(|r| r.as_str())).unwrap();
        let acquire_content =
            RegexSet::new(options.ingest_content_regexs.iter().map(|r| r.as_str())).unwrap();
        let ce_rules = CapturableExecutableRegexRules::new(
            Some(&options.capturable_executables_regexs),
            Some(&options.captured_exec_sql_regexs),
        )
        .unwrap();

        ResourceCollection {
            walked,
            ignore_paths_regex_set: ignore_paths,
            ingest_content_regex_set: acquire_content,
            ce_rules,
            ur_builder: UniformResourceBuilder {
                nature_bind: options.nature_bind.clone(),
            },
        }
    }

    // create a physical file system mapped via VFS, mainly for testing and experimental use
    pub fn from_vfs_physical_fs(
        fs_root_paths: &[String],
        options: &ResourceCollectionOptions,
    ) -> ResourceCollection {
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

        ResourceCollection::new(vfs_iter.map(EncounterableResource::Vfs).collect(), options)
    }

    // create a ignore::Walk instance which is a "smart" ignore because it honors .gitigore and .ignore
    // files in the walk path as well as the ignore and other directives passed in via options
    pub fn from_smart_ignore(
        fs_root_paths: &[String],
        options: &ResourceCollectionOptions,
        include_hidden: bool,
    ) -> ResourceCollection {
        let vfs_iter = fs_root_paths.iter().flat_map(move |root_path| {
            let ignorable_walk = if include_hidden {
                ignore::WalkBuilder::new(root_path).hidden(false).build()
            } else {
                ignore::Walk::new(root_path)
            };
            ignorable_walk.into_iter().flatten()
        });

        ResourceCollection::new(
            vfs_iter.map(EncounterableResource::SmartIgnore).collect(),
            options,
        )
    }

    // create a traditional walkdir::WalkDir which only ignore files based on file names rules passed in
    pub fn from_walk_dir(
        fs_root_paths: &[String],
        options: &ResourceCollectionOptions,
    ) -> ResourceCollection {
        let vfs_iter = fs_root_paths
            .iter()
            .flat_map(move |root_path| walkdir::WalkDir::new(root_path).into_iter().flatten());

        ResourceCollection::new(
            vfs_iter.map(EncounterableResource::WalkDir).collect(),
            options,
        )
    }

    pub fn ignored(&self) -> impl Iterator<Item = &EncounterableResource> + '_ {
        self.walked
            .iter()
            .filter(|er| self.ignore_paths_regex_set.is_match(&er.uri()))
    }

    pub fn not_ignored(&self) -> impl Iterator<Item = &EncounterableResource> + '_ {
        self.walked
            .iter()
            .filter(|er| !self.ignore_paths_regex_set.is_match(&er.uri()))
    }

    pub fn encountered_resources(
        &self,
    ) -> impl Iterator<Item = EncounteredResource<ContentResource>> + '_ {
        self.walked.iter().map(move |er| {
            let uri = er.uri();
            let ero = EncounterableResourceOptions {
                is_ignored: self.ignore_paths_regex_set.is_match(&uri),
                acquire_content: self.ingest_content_regex_set.is_match(&uri),
                capturable_executable: er.capturable_executable(&self.ce_rules),
            };
            er.encountered_resource(&ero)
        })
    }

    pub fn capturable_executables(&self) -> impl Iterator<Item = CapturableExecutable> + '_ {
        self.walked
            .iter()
            // "smart" means to try the path name and ensure that file is executable on disk
            .filter_map(|rwe| rwe.capturable_executable(&self.ce_rules))
    }

    pub fn uniform_resources(
        &self,
    ) -> impl Iterator<Item = anyhow::Result<UniformResource<ContentResource>, Box<dyn Error>>> + '_
    {
        self.encountered_resources()
            .filter_map(move |crs| match crs {
                EncounteredResource::Resource(resource) => {
                    match self.ur_builder.uniform_resource(resource) {
                        Ok(uniform_resource) => Some(Ok(*uniform_resource)),
                        Err(e) => Some(Err(e)), // error will be returned
                    }
                }
                EncounteredResource::Ignored(_)
                | EncounteredResource::NotFile(_)
                | EncounteredResource::NotFound(_) => None, // these will be filtered via `filter_map`
            })
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
