use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{Error as IoError, Read, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use is_executable::IsExecutable;
use regex::RegexSet;
use sha1::{Digest, Sha1};
use walkdir::WalkDir;

use crate::frontmatter::frontmatter;
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

#[derive(Debug, Clone)]
pub struct FileBinaryContent {
    hash: String,
    binary: Vec<u8>,
}

impl BinaryContent for FileBinaryContent {
    fn content_digest_hash(&self) -> &str {
        &self.hash
    }

    fn content_binary(&self) -> &Vec<u8> {
        &self.binary
    }
}

#[derive(Debug, Clone)]
pub struct FileTextContent {
    pub hash: String,
    pub text: String,
}

impl TextContent for FileTextContent {
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

pub type FileSysResourceQualifier = Box<dyn Fn(&Path, &str, &fs::File) -> bool>;
pub type FileSysExecutableQualifier =
    Box<dyn Fn(&Path, &str, &fs::File) -> Option<CapturableExecutable>>;

// Implementing the main logic
pub struct FileSysResourceSupplier {
    is_resource_ignored: FileSysResourceQualifier,
    is_content_available: FileSysResourceQualifier,
    is_capturable_executable: FileSysExecutableQualifier,
}

impl FileSysResourceSupplier {
    pub fn new(
        is_resource_ignored: FileSysResourceQualifier,
        is_content_available: FileSysResourceQualifier,
        is_capturable_executable: FileSysExecutableQualifier,
    ) -> Self {
        Self {
            is_resource_ignored,
            is_content_available,
            is_capturable_executable,
        }
    }
}

impl ContentResourceSupplier<ContentResource> for FileSysResourceSupplier {
    fn content_resource(&self, uri: &str) -> ContentResourceSupplied<ContentResource> {
        let path = match std::fs::canonicalize(uri) {
            Ok(p) => p,
            Err(_) => return ContentResourceSupplied::NotFound(uri.to_string()),
        };
        let path = &path;

        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return ContentResourceSupplied::NotFound(uri.to_string()),
        };

        if !metadata.is_file() {
            return ContentResourceSupplied::NotFile(uri.to_string());
        }

        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(_) => return ContentResourceSupplied::Error(Box::new(IoError::last_os_error())),
        };

        let nature = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if (self.is_resource_ignored)(path, &nature, &file) {
            return ContentResourceSupplied::Ignored(uri.to_string());
        }

        let file_size = metadata.len();
        let created_at = metadata.created().ok().map(DateTime::<Utc>::from);
        let last_modified_at = metadata.modified().ok().map(DateTime::<Utc>::from);
        let content_binary_supplier: Option<BinaryContentSupplier>;
        let content_text_supplier: Option<TextContentSupplier>;
        let capturable_executable: Option<CapturableExecutable>;
        let capturable_exec_binary_supplier: Option<BinaryExecOutputSupplier>;
        let capturable_exec_text_supplier: Option<TextExecOutputSupplier>;

        if let Some(capturable) = (self.is_capturable_executable)(path, &nature, &file) {
            capturable_executable = Some(capturable.clone());

            if !matches!(capturable, CapturableExecutable::RequestedButNotExecutable) {
                let uri_clone_cebs = uri.to_string(); // Clone for the first closure
                capturable_exec_binary_supplier = Some(Box::new(
                    move |stdin| -> Result<BinaryExecOutput, Box<dyn Error>> {
                        let mut exec = subprocess::Exec::cmd(&uri_clone_cebs)
                            .stdout(subprocess::Redirection::Pipe);

                        if stdin.is_some() {
                            exec = exec.stdin(subprocess::Redirection::Pipe);
                        }

                        let mut popen = exec.popen()?;

                        if let Some(stdin_text) = stdin {
                            if let Some(mut stdin_pipe) = popen.stdin.take() {
                                stdin_pipe.write_all(stdin_text.as_bytes())?;
                                stdin_pipe.flush()?;
                                // `stdin_pipe` is dropped here when it goes out of scope, closing the stdin of the subprocess
                            } // else: no one is listening to the stdin of the subprocess, so we can't pipe anything to it
                        }

                        let status = popen.wait()?;

                        let mut output = popen.stdout.take().unwrap();
                        let mut binary = Vec::new();
                        output.read_to_end(&mut binary)?;

                        let mut error_output = String::new();
                        match &mut popen.stderr.take() {
                            Some(stderr) => {
                                stderr.read_to_string(&mut error_output)?;
                            }
                            None => {}
                        }

                        let hash = {
                            let mut hasher = Sha1::new();
                            hasher.update(&binary);
                            format!("{:x}", hasher.finalize())
                        };

                        Ok((
                            Box::new(FileBinaryContent { hash, binary }) as Box<dyn BinaryContent>,
                            status,
                            if !error_output.is_empty() {
                                Some(error_output)
                            } else {
                                None
                            },
                        ))
                    },
                ));

                let uri_clone_cets = uri.to_string(); // Clone for the second closure
                capturable_exec_text_supplier = Some(Box::new(
                    move |stdin| -> Result<TextExecOutput, Box<dyn Error>> {
                        let mut exec = subprocess::Exec::cmd(&uri_clone_cets)
                            .stdout(subprocess::Redirection::Pipe);

                        if stdin.is_some() {
                            exec = exec.stdin(subprocess::Redirection::Pipe);
                        }

                        let mut popen = exec.popen()?;

                        if let Some(stdin_text) = stdin {
                            if let Some(mut stdin_pipe) = popen.stdin.take() {
                                stdin_pipe.write_all(stdin_text.as_bytes())?;
                                stdin_pipe.flush()?;
                                // `stdin_pipe` is dropped here when it goes out of scope, closing the stdin of the subprocess
                            } // else: no one is listening to the stdin of the subprocess, so we can't pipe anything to it
                        }

                        let status = popen.wait()?;

                        let mut output = String::new();
                        popen.stdout.take().unwrap().read_to_string(&mut output)?;

                        let mut error_output = String::new();
                        match &mut popen.stderr.take() {
                            Some(stderr) => {
                                stderr.read_to_string(&mut error_output)?;
                            }
                            None => {}
                        }

                        let hash = {
                            let mut hasher = Sha1::new();
                            hasher.update(output.as_bytes());
                            format!("{:x}", hasher.finalize())
                        };

                        Ok((
                            Box::new(FileTextContent { hash, text: output })
                                as Box<dyn TextContent>,
                            status,
                            if !error_output.is_empty() {
                                Some(error_output)
                            } else {
                                None
                            },
                        ))
                    },
                ));
            } else {
                capturable_exec_binary_supplier = None;
                capturable_exec_text_supplier = None;
            }
        } else {
            capturable_executable = None;
            capturable_exec_binary_supplier = None;
            capturable_exec_text_supplier = None;
        }

        if (self.is_content_available)(path, &nature, &file) {
            let uri_clone_cbs = uri.to_string(); // Clone for the first closure
            content_binary_supplier = Some(Box::new(
                move || -> Result<Box<dyn BinaryContent>, Box<dyn Error>> {
                    let mut binary = Vec::new();
                    let mut file = fs::File::open(&uri_clone_cbs)?;
                    file.read_to_end(&mut binary)?;

                    let hash = {
                        let mut hasher = Sha1::new();
                        hasher.update(&binary);
                        format!("{:x}", hasher.finalize())
                    };

                    Ok(Box::new(FileBinaryContent { hash, binary }) as Box<dyn BinaryContent>)
                },
            ));

            let uri_clone_cts = uri.to_string(); // Clone for the second closure
            content_text_supplier = Some(Box::new(
                move || -> Result<Box<dyn TextContent>, Box<dyn Error>> {
                    let mut text = String::new();
                    let mut file = fs::File::open(&uri_clone_cts)?;
                    file.read_to_string(&mut text)?;

                    let hash = {
                        let mut hasher = Sha1::new();
                        hasher.update(&text);
                        format!("{:x}", hasher.finalize())
                    };

                    Ok(Box::new(FileTextContent { hash, text }) as Box<dyn TextContent>)
                },
            ));
        } else {
            content_binary_supplier = None;
            content_text_supplier = None;
        }

        ContentResourceSupplied::Resource(ContentResource {
            uri: String::from(path.to_str().unwrap()),
            nature: Some(nature),
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
}

pub struct FileSysUniformResourceSupplier;

impl UniformResourceSupplier<ContentResource> for FileSysUniformResourceSupplier {
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
        if let Some(nature) = &resource.nature {
            match nature.as_str() {
                // Match different file extensions
                "html" => {
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
                "json" | "jsonc" => {
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
                "yaml" | "yml" => {
                    let yaml = YamlResource {
                        resource,
                        content: None, // TODO parse using serde
                    };
                    Ok(Box::new(UniformResource::Yaml(yaml)))
                }
                "toml" => {
                    let toml = TomlResource {
                        resource,
                        content: None, // TODO parse using serde
                    };
                    Ok(Box::new(UniformResource::Toml(toml)))
                }
                "md" | "mdx" => {
                    let markdown = MarkdownResource { resource };
                    Ok(Box::new(UniformResource::Markdown(markdown)))
                }
                "png" | "gif" | "tiff" | "jpg" | "jpeg" => {
                    let image = ImageResource {
                        resource,
                        image_meta: HashMap::new(), // TODO add meta data
                    };
                    Ok(Box::new(UniformResource::Image(image)))
                }
                "svg" => {
                    let svg = SvgResource { resource };
                    Ok(Box::new(UniformResource::Svg(svg)))
                }
                "tap" => {
                    let tap = TestAnythingResource { resource };
                    Ok(Box::new(UniformResource::Tap(tap)))
                }
                _ => Ok(Box::new(UniformResource::Unknown(resource))),
            }
        } else {
            Err("Unknown resource nature.".into())
        }
    }
}

pub struct FileSysResourcesWalker {
    pub root_paths: Vec<String>,
    pub resource_supplier: FileSysResourceSupplier,
    pub uniform_resource_supplier: FileSysUniformResourceSupplier,
}

impl FileSysResourcesWalker {
    pub fn new(
        root_paths: &[String],
        ignore_paths_regexs: &[regex::Regex],
        inspect_content_regexs: &[regex::Regex],
        capturable_executables_regexs: &[regex::Regex],
        captured_exec_sql_regexs: &[regex::Regex],
    ) -> Result<Self, regex::Error> {
        // Constructor can fail due to RegexSet::new
        let ignore_paths = RegexSet::new(ignore_paths_regexs.iter().map(|r| r.as_str()))?;
        let inspect_content_paths =
            RegexSet::new(inspect_content_regexs.iter().map(|r| r.as_str()))?;
        let capturable_executables = capturable_executables_regexs.to_vec();
        let captured_exec_sql = RegexSet::new(captured_exec_sql_regexs.iter().map(|r| r.as_str()))?;

        let resource_supplier = FileSysResourceSupplier::new(
            Box::new(move |path, _nature, _file| {
                let abs_path = path.to_str().unwrap();
                ignore_paths.is_match(abs_path)
            }),
            Box::new(move |path, _nature, _file| {
                inspect_content_paths.is_match(path.to_str().unwrap())
            }),
            Box::new(move |path, _nature, _file| {
                let mut ce: Option<CapturableExecutable> = None;
                let haystack = path.to_str().unwrap();

                if captured_exec_sql.is_match(haystack) {
                    ce = Some(CapturableExecutable::Text(
                        String::from("surveilr-SQL"),
                        true,
                    ));
                } else {
                    for re in capturable_executables.iter() {
                        if let Some(caps) = re.captures(haystack) {
                            if let Some(nature) = caps.name("nature") {
                                ce = Some(CapturableExecutable::Text(
                                    String::from(nature.as_str()),
                                    false,
                                ));
                                break;
                            } else {
                                ce = Some(CapturableExecutable::RequestedButNoNature(re.clone()));
                                break;
                            }
                        }
                    }
                }
                if ce.is_some() {
                    if path.is_executable() {
                        return ce;
                    } else {
                        return Some(CapturableExecutable::RequestedButNotExecutable);
                    }
                }
                None
            }),
        );

        let uniform_resource_supplier = FileSysUniformResourceSupplier {};

        Ok(Self {
            root_paths: root_paths.to_owned(),
            resource_supplier,
            uniform_resource_supplier,
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
                        match self.uniform_resource_supplier.uniform_resource(resource) {
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
                            match self.uniform_resource_supplier.uniform_resource(resource) {
                                Ok(uniform_resource) => {
                                    Some(Ok((entry.clone(), *uniform_resource)))
                                }
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

// For the unit test, we use the built-in testing framework
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesys_resource_supplier() {
        // Set up a FileSysResourceSupplier with mock callbacks
        let supplier = FileSysResourceSupplier::new(
            Box::new(|_file, _nature, _metadata| false), // is_resource_ignored
            Box::new(|_file, _nature, _metadata| true),  // is_content_available
            Box::new(|_file, _nature, _metadata| None),  // is_capturable_executable
        );

        // Create a file for the test environment, writing some content
        let test_file_path = "test.txt";
        let test_data = b"Hello, world!";
        fs::write(test_file_path, test_data).expect("Unable to write test file");

        // Use the supplier to get a resource
        let result = supplier.content_resource(test_file_path);

        match result {
            ContentResourceSupplied::Resource(res) => {
                let cbin =
                    (res.content_binary_supplier.unwrap())().expect("Error obtaining content");
                assert_eq!(cbin.content_binary(), b"Hello, world!");
                let ctext =
                    (res.content_text_supplier.unwrap())().expect("Error obtaining content");
                assert_eq!(ctext.content_text(), "Hello, world!");
            }
            _ => panic!("Unexpected result from resource()"),
        }

        // Clean up the file system
        fs::remove_file(test_file_path).expect("Unable to delete test file");
    }
}
