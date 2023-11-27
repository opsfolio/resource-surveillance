use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use regex::RegexSet;
use walkdir::WalkDir;

use crate::capturable::*;
use crate::fscontent::*;
use crate::resource::*;

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

// Implementing the main logic
pub struct FileSysResourceSupplier {
    pub fspc_options: FileSysPathContentOptions,
    pub ur_builder: UniformResourceBuilder,
}

impl FileSysResourceSupplier {
    pub fn new(
        is_resource_ignored: FileSysPathQualifier,
        is_content_available: FileSysPathQualifier,
        is_capturable_executable: FileSysPathCapExecQualifier,
        nature_bind: &HashMap<String, String>,
    ) -> Self {
        Self {
            fspc_options: FileSysPathContentOptions {
                is_ignored: FileSysPathOption::Check(is_resource_ignored),
                has_content: FileSysPathOption::Check(is_content_available),
                is_capturable_executable: Some(is_capturable_executable),
            },
            ur_builder: UniformResourceBuilder {
                nature_bind: nature_bind.clone(),
            },
        }
    }
}

impl ContentResourceSupplier<ContentResource> for FileSysResourceSupplier {
    fn content_resource(&self, uri: &str) -> ContentResourceSupplied<ContentResource> {
        fs_path_content_resource(uri, &self.fspc_options)
    }
}

impl UniformResourceSupplier<ContentResource> for FileSysResourceSupplier {
    fn uniform_resource(
        &self,
        resource: ContentResource,
    ) -> Result<Box<UniformResource<ContentResource>>, Box<dyn Error>> {
        self.ur_builder.uniform_resource(resource)
    }
}

pub struct FileSysResourcesWalker {
    pub root_paths: Vec<String>,
    pub resource_supplier: FileSysResourceSupplier,
}

impl FileSysResourcesWalker {
    pub fn new(
        root_paths: &[String],
        ignore_paths_regexs: &[regex::Regex],
        inspect_content_regexs: &[regex::Regex],
        capturable_executables_regexs: &[regex::Regex],
        captured_exec_sql_regexs: &[regex::Regex],
        nature_bind: &HashMap<String, String>,
    ) -> Result<Self, regex::Error> {
        // Constructor can fail due to RegexSet::new
        let ignore_paths = RegexSet::new(ignore_paths_regexs.iter().map(|r| r.as_str()))?;
        let inspect_content_paths =
            RegexSet::new(inspect_content_regexs.iter().map(|r| r.as_str()))?;
        let cerr = CapturableExecutableRegexRules::new(
            Some(capturable_executables_regexs),
            Some(captured_exec_sql_regexs),
        )
        .unwrap();

        let resource_supplier = FileSysResourceSupplier::new(
            Box::new(move |path, _nature, _file| {
                let abs_path = path.to_str().unwrap();
                ignore_paths.is_match(abs_path)
            }),
            Box::new(move |path, _nature, _file| {
                inspect_content_paths.is_match(path.to_str().unwrap())
            }),
            Box::new(move |path, _nature, _file| cerr.path_capturable_executable(path)),
            nature_bind,
        );

        Ok(Self {
            root_paths: root_paths.to_owned(),
            resource_supplier,
        })
    }

    pub fn _walk_resources<F>(&self, mut encounter_resource: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(UniformResource<ContentResource>) + 'static,
    {
        for root in &self.root_paths {
            // Walk through each entry in the directory.
            for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                let uri = entry.path().to_string_lossy().into_owned();

                // Use the ResourceSupplier to create a resource from the file.
                match self.resource_supplier.content_resource(&uri) {
                    ContentResourceSupplied::Resource(resource) => {
                        // Create a uniform resource for each valid resource.
                        match self.resource_supplier.uniform_resource(resource) {
                            Ok(uniform_resource) => encounter_resource(*uniform_resource),
                            Err(e) => return Err(e), // Handle error according to your policy
                        }
                    }
                    ContentResourceSupplied::Error(e) => return Err(e),
                    ContentResourceSupplied::Ignored(_) => {}
                    ContentResourceSupplied::NotFile(_) => {}
                    ContentResourceSupplied::NotFound(_) => {} // TODO: should this be an error?
                }
            }
        }

        Ok(())
    }

    pub fn walk_resources_iter(
        &self,
    ) -> impl Iterator<
        Item = Result<(walkdir::DirEntry, UniformResource<ContentResource>), Box<dyn Error>>,
    > + '_ {
        self.root_paths.iter().flat_map(move |root| {
            WalkDir::new(root)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter_map(move |entry| {
                    let uri = entry.path().to_string_lossy().into_owned();
                    match self.resource_supplier.content_resource(&uri) {
                        ContentResourceSupplied::Resource(resource) => {
                            match self.resource_supplier.uniform_resource(resource) {
                                Ok(uniform_resource) => Some(Ok((entry, *uniform_resource))),
                                Err(e) => Some(Err(e)),
                            }
                        }
                        ContentResourceSupplied::Error(e) => Some(Err(e)),
                        ContentResourceSupplied::Ignored(_)
                        | ContentResourceSupplied::NotFile(_)
                        | ContentResourceSupplied::NotFound(_) => None,
                    }
                })
        })
    }
}
