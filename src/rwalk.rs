use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::canonicalize;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use regex::RegexSet;
use sha1::{Digest, Sha1};

use crate::capturable::*;
use crate::frontmatter::frontmatter;
use crate::resource::*;
use crate::subprocess::*;

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
pub struct ResourceContentOptions {
    pub is_physical_fs: bool,
    pub is_ignored: bool,
    pub acquire_content: bool,
    pub capturable_executable: Option<CapturableExecutable>,
}

#[derive(Debug)]
pub struct ResourceContentMetaData {
    pub is_file: bool,
    pub is_dir: bool,
    pub file_size: u64,
    pub created_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>,
    pub last_modified_at: Option<chrono::prelude::DateTime<chrono::prelude::Utc>>,
}

impl ResourceContentMetaData {
    pub fn from_fs_path(fs_path: &Path) -> anyhow::Result<ResourceContentMetaData> {
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

        Ok(ResourceContentMetaData {
            is_file,
            is_dir,
            file_size,
            created_at,
            last_modified_at,
        })
    }

    pub fn from_vfs_path(vfs_path: &vfs::VfsPath) -> anyhow::Result<ResourceContentMetaData> {
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

        Ok(ResourceContentMetaData {
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
        options: &ResourceContentOptions,
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
        options: &ResourceContentOptions,
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

pub enum ResourceWalkerEntry {
    WalkDir(walkdir::DirEntry),
    SmartIgnore(ignore::DirEntry),
    Vfs(vfs::VfsPath),
}

impl ResourceWalkerEntry {
    fn uri(&self) -> String {
        match self {
            ResourceWalkerEntry::WalkDir(de) => de.path().to_string_lossy().to_string(),
            ResourceWalkerEntry::SmartIgnore(de) => de.path().to_string_lossy().to_string(),
            ResourceWalkerEntry::Vfs(path) => path.as_str().to_string(),
        }
    }

    fn _path(&self) -> Option<&Path> {
        match self {
            ResourceWalkerEntry::WalkDir(de) => Some(de.path()),
            ResourceWalkerEntry::SmartIgnore(de) => Some(de.path()),
            ResourceWalkerEntry::Vfs(_path) => None,
        }
    }

    pub fn meta_data(&self) -> anyhow::Result<ResourceContentMetaData> {
        match self {
            ResourceWalkerEntry::WalkDir(de) => ResourceContentMetaData::from_fs_path(de.path()),
            ResourceWalkerEntry::SmartIgnore(de) => {
                ResourceContentMetaData::from_fs_path(de.path())
            }
            ResourceWalkerEntry::Vfs(path) => ResourceContentMetaData::from_vfs_path(path),
        }
    }

    pub fn content_suppliers(&self, options: &ResourceContentOptions) -> ResourceContentSuppliers {
        match self {
            ResourceWalkerEntry::WalkDir(de) => {
                ResourceContentSuppliers::from_fs_path(de.path(), options)
            }
            ResourceWalkerEntry::SmartIgnore(de) => {
                ResourceContentSuppliers::from_fs_path(de.path(), options)
            }
            ResourceWalkerEntry::Vfs(path) => {
                ResourceContentSuppliers::from_vfs_path(path, options)
            }
        }
    }

    pub fn capturable_executable(
        &self,
        ce_rules: &CapturableExecutableRegexRules,
    ) -> Option<CapturableExecutable> {
        match self {
            ResourceWalkerEntry::WalkDir(de) => ce_rules.path_capturable_executable(de.path()),
            ResourceWalkerEntry::SmartIgnore(de) => ce_rules.path_capturable_executable(de.path()),
            ResourceWalkerEntry::Vfs(path) => ce_rules.uri_capturable_executable(path.as_str()),
        }
    }

    pub fn resource_content(
        &self,
        options: &ResourceContentOptions,
    ) -> ContentResourceSupplied<ContentResource> {
        let uri = self.uri();
        if options.is_ignored {
            return ContentResourceSupplied::Ignored(uri);
        }

        let metadata = match self.meta_data() {
            Ok(metadata) => {
                if !metadata.is_file {
                    return ContentResourceSupplied::NotFile(uri);
                }
                metadata
            }
            Err(_) => return ContentResourceSupplied::NotFound(uri),
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

        ContentResourceSupplied::Resource(ContentResource {
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
pub struct ResourceWalker {
    pub physical_fs_root_paths: Vec<String>,
}

pub struct ResourceCollectionOptions {
    pub ignore_paths_regexs: Vec<regex::Regex>,
    pub acquire_content_regexs: Vec<regex::Regex>,
    pub capturable_executables_regexs: Vec<regex::Regex>,
    pub captured_exec_sql_regexs: Vec<regex::Regex>,
    pub nature_bind: HashMap<String, String>,
}

pub struct ResourceCollection {
    pub walked: Vec<ResourceWalkerEntry>,
    pub ignore_paths: RegexSet,
    pub acquire_content: RegexSet,
    pub ce_rules: CapturableExecutableRegexRules,
    pub ur_builder: UniformResourceBuilder,
}

impl ResourceCollection {
    pub fn new(
        walked: Vec<ResourceWalkerEntry>,
        options: &ResourceCollectionOptions,
    ) -> ResourceCollection {
        let ignore_paths =
            RegexSet::new(options.ignore_paths_regexs.iter().map(|r| r.as_str())).unwrap();
        let acquire_content =
            RegexSet::new(options.acquire_content_regexs.iter().map(|r| r.as_str())).unwrap();
        let ce_rules = CapturableExecutableRegexRules::new(
            Some(&options.capturable_executables_regexs),
            Some(&options.captured_exec_sql_regexs),
        )
        .unwrap();

        ResourceCollection {
            walked,
            ignore_paths,
            acquire_content,
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

        ResourceCollection::new(vfs_iter.map(ResourceWalkerEntry::Vfs).collect(), options)
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
            vfs_iter.map(ResourceWalkerEntry::SmartIgnore).collect(),
            options,
        )
    }

    // create a traditional walkdir::WalkDir which only ignore files based on file names rules passed in
    pub fn from_walk_dir(
        fs_root_paths: &[String],
        options: &ResourceCollectionOptions,
    ) -> ResourceCollection {
        let vfs_iter = fs_root_paths.iter().flat_map(move |fs_root_path| {
            walkdir::WalkDir::new(fs_root_path).into_iter().flatten()
        });

        ResourceCollection::new(
            vfs_iter.map(ResourceWalkerEntry::WalkDir).collect(),
            options,
        )
    }

    pub fn ignored(&self) -> impl Iterator<Item = &ResourceWalkerEntry> + '_ {
        self.walked
            .iter()
            .filter(|rwe| self.ignore_paths.is_match(&rwe.uri()))
    }

    pub fn not_ignored(&self) -> impl Iterator<Item = &ResourceWalkerEntry> + '_ {
        self.walked
            .iter()
            .filter(|rwe| !self.ignore_paths.is_match(&rwe.uri()))
    }

    pub fn content_resources(
        &self,
    ) -> impl Iterator<Item = ContentResourceSupplied<ContentResource>> + '_ {
        self.walked.iter().map(move |rwe| {
            let uri = rwe.uri();
            let eco = ResourceContentOptions {
                is_ignored: self.ignore_paths.is_match(&uri),
                acquire_content: self.acquire_content.is_match(&uri),
                capturable_executable: rwe.capturable_executable(&self.ce_rules),
                is_physical_fs: true,
            };
            rwe.resource_content(&eco)
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
        self.content_resources().filter_map(move |crs| match crs {
            ContentResourceSupplied::Resource(resource) => {
                match self.ur_builder.uniform_resource(resource) {
                    Ok(uniform_resource) => Some(Ok(*uniform_resource)),
                    Err(e) => Some(Err(e)), // error will be returned
                }
            }
            ContentResourceSupplied::Error(e) => Some(Err(e)), // error will be returned
            ContentResourceSupplied::Ignored(_)
            | ContentResourceSupplied::NotFile(_)
            | ContentResourceSupplied::NotFound(_) => None, // these will be filtered via `filter_map`
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
