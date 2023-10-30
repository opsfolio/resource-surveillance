use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{Error as IoError, Read};
use std::path::Path;

use chrono::{DateTime, Utc};
use regex::RegexSet;
use sha1::{Digest, Sha1};
use walkdir::WalkDir;

use crate::resource::*;

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

pub struct FileTextContent {
    hash: String,
    text: String,
}

impl TextContent for FileTextContent {
    fn content_digest_hash(&self) -> &str {
        &self.hash
    }

    fn content_text(&self) -> &str {
        &self.text
    }
}

pub type FileSysResourceQualifier = Box<dyn Fn(&Path, &str, &fs::File) -> bool>;

// Implementing the main logic
pub struct FileSysResourceSupplier {
    is_resource_ignored: FileSysResourceQualifier,
    is_content_available: FileSysResourceQualifier,
}

impl FileSysResourceSupplier {
    pub fn new(
        is_resource_ignored: FileSysResourceQualifier,
        is_content_available: FileSysResourceQualifier,
    ) -> Self {
        Self {
            is_resource_ignored,
            is_content_available,
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
            content_binary_supplier,
            content_text_supplier,
        })
    }
}

pub struct FileSysUniformResourceSupplier;

impl UniformResourceSupplier<ContentResource> for FileSysUniformResourceSupplier {
    fn uniform_resource(
        &self,
        resource: ContentResource,
    ) -> Result<Box<UniformResource<ContentResource>>, Box<dyn Error>> {
        // Based on the nature of the resource, we determine the type of UniformResource
        if let Some(nature) = &resource.nature {
            match nature.as_str() {
                // Match different file extensions
                "html" => {
                    let html = HtmlResource {
                        resource,
                        head_meta: HashMap::new(),
                    };
                    Ok(Box::new(UniformResource::Html(html)))
                }
                "json" => {
                    let json = JsonResource {
                        resource,
                        content: None,
                    };
                    Ok(Box::new(UniformResource::Json(json)))
                }
                "md" | "mdx" => {
                    let markdown = MarkdownResource {
                        resource,
                        frontmatter: None,
                    };
                    Ok(Box::new(UniformResource::Markdown(markdown)))
                }
                "png" | "gif" | "tiff" | "jpg" | "jpeg" => {
                    let image = ImageResource {
                        resource,
                        image_meta: HashMap::new(),
                    };
                    Ok(Box::new(UniformResource::Image(image)))
                }
                _ => Ok(Box::new(UniformResource::Unknown(resource))),
            }
        } else {
            Err("Unknown resource nature.".into())
        }
    }
}

pub struct FileSysResourcesWalker {
    root_paths: Vec<String>,
    resource_supplier: FileSysResourceSupplier,
    uniform_resource_supplier: FileSysUniformResourceSupplier,
}

impl FileSysResourcesWalker {
    pub fn new(
        root_paths: &[String],
        ignore_paths_regexs: &[regex::Regex],
        inspect_content_regexs: &[regex::Regex],
    ) -> Result<Self, regex::Error> {
        // Constructor can fail due to RegexSet::new
        let ignore_paths = RegexSet::new(ignore_paths_regexs.iter().map(|r| r.as_str()))?;
        let inspect_content_paths =
            RegexSet::new(inspect_content_regexs.iter().map(|r| r.as_str()))?;

        let resource_supplier = FileSysResourceSupplier::new(
            Box::new(move |path, _nature, _file| ignore_paths.is_match(path.to_str().unwrap())),
            Box::new(move |path, _nature, _file| {
                inspect_content_paths.is_match(path.to_str().unwrap())
            }),
        );

        let uniform_resource_supplier = FileSysUniformResourceSupplier {};

        Ok(Self {
            root_paths: root_paths.to_owned(),
            resource_supplier,
            uniform_resource_supplier,
        })
    }

    pub fn walk_resources<F>(&self, mut encounter_resource: F) -> Result<(), Box<dyn Error>>
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

    // ... rest of your implementation
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
