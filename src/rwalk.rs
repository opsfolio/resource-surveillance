use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::canonicalize;
use std::path::Path;
use std::path::PathBuf;

use regex::RegexSet;
use sha1::{Digest, Sha1};
use std::collections::VecDeque;
use vfs::{FileSystem as VirtualFileSystem, PhysicalFS, VfsFileType, VfsPath};

use crate::capturable::*;
use crate::frontmatter::frontmatter;
use crate::resource::*;
use crate::subprocess::*;

type VfsIdentity = String;

lazy_static::lazy_static! {
    pub static ref VFS_PHYSICAL_CWD: PhysicalFS = PhysicalFS::new(".");
    pub static ref VFS_CATALOG: HashMap<VfsIdentity, &'static dyn VirtualFileSystem> = HashMap::default();
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
pub struct ResourceContentOptions {
    pub is_physical_fs: bool,
    pub is_ignored: bool,
    pub acquire_content: bool,
    pub capturable_executable: Option<CapturableExecutable>,
}

pub fn resource_content(
    path: &VfsPath,
    options: &ResourceContentOptions,
) -> ContentResourceSupplied<ContentResource> {
    let uri = path.as_str();
    if options.is_ignored {
        return ContentResourceSupplied::Ignored(uri.to_string());
    }

    let metadata = match path.metadata() {
        Ok(metadata) => metadata,
        Err(_) => return ContentResourceSupplied::NotFound(uri.to_string()),
    };

    match metadata.file_type {
        vfs::VfsFileType::File => {}
        vfs::VfsFileType::Directory => return ContentResourceSupplied::NotFile(uri.to_string()),
    };

    // typically the nature is a the file's extension
    let nature = uri.rsplit_once('.').map(|(_, ext)| ext.to_string());

    let file_size = metadata.len;
    let created_at; // TODO; figure out how to get the created stamp
    let last_modified_at; // TODO; figure out how to get the created stamp
    let content_binary_supplier: Option<BinaryContentSupplier>;
    let content_text_supplier: Option<TextContentSupplier>;
    let capturable_executable: Option<CapturableExecutable>;
    let capturable_exec_binary_supplier: Option<BinaryExecOutputSupplier>;
    let capturable_exec_text_supplier: Option<TextExecOutputSupplier>;

    if options.is_physical_fs {
        let fs_path = std::path::Path::new(uri);
        match fs::metadata(fs_path) {
            Ok(metadata) => {
                created_at = metadata
                    .created()
                    .ok()
                    .map(chrono::DateTime::<chrono::Utc>::from);
                last_modified_at = metadata
                    .modified()
                    .ok()
                    .map(chrono::DateTime::<chrono::Utc>::from);
            }
            Err(_) => {
                created_at = None;
                last_modified_at = None;
            }
        }
    } else {
        created_at = None;
        last_modified_at = None;
    }

    if let Some(capturable) = &options.capturable_executable {
        capturable_executable = Some(capturable.clone());
        capturable_exec_binary_supplier = capturable.executable_content_binary();
        capturable_exec_text_supplier = capturable.executable_content_text();
    } else {
        capturable_executable = None;
        capturable_exec_binary_supplier = None;
        capturable_exec_text_supplier = None;
    }

    if options.acquire_content {
        let path_clone_cbs = path.clone();
        content_binary_supplier = Some(Box::new(
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

        let path_clone_cts = path.clone();
        content_text_supplier = Some(Box::new(
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
        content_binary_supplier = None;
        content_text_supplier = None;
    }

    ContentResourceSupplied::Resource(ContentResource {
        uri: uri.to_string(),
        nature,
        size: Some(file_size),
        created_at,
        last_modified_at,
        capturable_executable,
        content_binary_supplier,
        content_text_supplier,
        capturable_exec_binary_supplier,
        capturable_exec_text_supplier,
    })
}

#[derive(Debug)]
pub struct ResourceWalkerOptions {
    pub physical_fs_root_paths: Vec<String>,
    pub ignore_paths_regexs: Vec<regex::Regex>,
    pub acquire_content_regexs: Vec<regex::Regex>,
    pub capturable_executables_regexs: Vec<regex::Regex>,
    pub captured_exec_sql_regexs: Vec<regex::Regex>,
    pub nature_bind: HashMap<String, String>,
}

pub struct ResourceWalker {
    pub physical_fs_root_paths: Vec<String>,
    pub ignore_paths: RegexSet,
    pub acquire_content: RegexSet,
    pub ce_rules: CapturableExecutableRegexRules,
    pub ur_builder: UniformResourceBuilder,
}

impl ResourceWalker {
    pub fn new(options: &ResourceWalkerOptions) -> ResourceWalker {
        let ignore_paths =
            RegexSet::new(options.ignore_paths_regexs.iter().map(|r| r.as_str())).unwrap();
        let acquire_content =
            RegexSet::new(options.acquire_content_regexs.iter().map(|r| r.as_str())).unwrap();
        let ce_rules = CapturableExecutableRegexRules::new(
            Some(&options.capturable_executables_regexs),
            Some(&options.captured_exec_sql_regexs),
        )
        .unwrap();

        ResourceWalker {
            physical_fs_root_paths: options.physical_fs_root_paths.clone(),
            ignore_paths,
            acquire_content,
            ce_rules,
            ur_builder: UniformResourceBuilder {
                nature_bind: options.nature_bind.clone(),
            },
        }
    }

    pub fn all(&self) -> impl Iterator<Item = VfsPath> + '_ {
        let mut vfs_walk_dirs: Vec<WalkPath> = vec![];
        for root_path in &self.physical_fs_root_paths {
            vfs_walk_dirs.push(WalkPath::physical(root_path));
        }

        vfs_walk_dirs.into_iter().flatten()
    }

    pub fn ignored(&self) -> impl Iterator<Item = VfsPath> + '_ {
        self.all()
            .filter(|vp| self.ignore_paths.is_match(vp.as_str()))
    }

    pub fn not_ignored(&self) -> impl Iterator<Item = VfsPath> + '_ {
        self.all()
            .filter(|vp| !self.ignore_paths.is_match(vp.as_str()))
    }

    pub fn content_resources(
        &self,
    ) -> impl Iterator<Item = ContentResourceSupplied<ContentResource>> + '_ {
        self.all().map(move |vp| {
            let vp_str = vp.as_str();
            let eco = ResourceContentOptions {
                is_ignored: self.ignore_paths.is_match(vp_str),
                acquire_content: self.acquire_content.is_match(vp_str),
                // "smart" means to try the path name and ensure that file is executable on disk
                capturable_executable: self.ce_rules.smart_path_capturable_executable(vp_str),
                is_physical_fs: true,
            };
            resource_content(&vp, &eco)
        })
    }

    pub fn capturable_executables(&self) -> impl Iterator<Item = CapturableExecutable> + '_ {
        self.all()
            // "smart" means to try the path name and ensure that file is executable on disk
            .filter_map(|vp| self.ce_rules.smart_path_capturable_executable(vp.as_str()))
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

// TODO vfs library supports path.walk_dir so see if it's worth using that instead

pub struct WalkPath {
    pub physical_fs_root_path: String,
    pub vfs_fs_root_path: VfsPath,
    pub to_visit: VecDeque<VfsPath>,
    pub max_depth: Option<usize>,
    pub current_depth: usize,
}

impl WalkPath {
    pub fn physical(physical_fs_root_path_orig: &str) -> Self {
        let physical_fs_root_path: String;
        if let Ok(canonical) = canonicalize(physical_fs_root_path_orig) {
            physical_fs_root_path = canonical.to_string_lossy().to_string();
        } else {
            eprintln!(
                "Error canonicalizing {}, trying original",
                physical_fs_root_path_orig
            );
            physical_fs_root_path = physical_fs_root_path_orig.to_string();
        }
        let vfs_fs_root = VfsPath::new(PhysicalFS::new("/"));
        let mut to_visit = VecDeque::new();
        to_visit.push_back(vfs_fs_root.join(physical_fs_root_path.clone()).unwrap());

        WalkPath {
            physical_fs_root_path,
            vfs_fs_root_path: vfs_fs_root,
            to_visit,
            max_depth: None,
            current_depth: 0,
        }
    }

    #[allow(dead_code)]
    pub fn max_depth(&mut self, depth: usize) -> &mut Self {
        self.max_depth = Some(depth);
        self
    }

    #[allow(dead_code)]
    pub fn build(self) -> WalkPath {
        self
    }
}

impl Iterator for WalkPath {
    type Item = VfsPath;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.to_visit.pop_front() {
            if let Ok(metadata) = path.metadata() {
                let is_dir = matches!(metadata.file_type, VfsFileType::Directory);

                if is_dir {
                    // Increment depth
                    self.current_depth += 1;

                    // Check for depth limit and whether we should descend into this directory
                    let should_descend =
                        self.max_depth.map_or(true, |max| self.current_depth <= max);

                    if should_descend {
                        if let Ok(entries) = path.read_dir() {
                            for entry in entries {
                                self.to_visit.push_back(entry);
                            }
                        }
                    }

                    // Decrement depth
                    self.current_depth -= 1;

                    // If not descending, return this directory path
                    if !should_descend {
                        return Some(path);
                    }
                } else {
                    // It's a file or a non-directory path, return it
                    return Some(path);
                }
            } else {
                // If metadata can't be obtained, still return the path
                return Some(path);
            }
        }

        // If the loop exits because `to_visit` is empty, return None
        None
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
