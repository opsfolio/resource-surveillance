use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{Error as IoError, Read};
use std::path::Path;

use chrono::{DateTime, Utc};
// use regex::RegexSet;
use sha1::{Digest, Sha1};
use walkdir::WalkDir;

use crate::resource::*;
use crate::uniform::*;

pub struct BinaryContent {
    hash: String,
    binary: Vec<u8>,
    text: String,
}

impl ResourceContent<Vec<u8>> for BinaryContent {
    fn content_digest_hash(&self, _target: Vec<u8>) -> &str {
        &self.hash
    }

    fn content_binary(&self, _target: Vec<u8>) -> &Vec<u8> {
        &self.binary
    }

    fn content_text(&self, _target: Vec<u8>) -> &str {
        &self.text
    }
}

// Implementing the main logic
pub struct FileSysResourceSupplier {
    is_resource_ignored: Box<dyn Fn(&fs::File, &str, &fs::Metadata) -> bool>,
    is_content_available: Box<dyn Fn(&fs::File, &str, &fs::Metadata) -> bool>,
}

impl FileSysResourceSupplier {
    pub fn new(
        is_resource_ignored: Box<dyn Fn(&fs::File, &str, &fs::Metadata) -> bool>,
        is_content_available: Box<dyn Fn(&fs::File, &str, &fs::Metadata) -> bool>,
    ) -> Self {
        Self {
            is_resource_ignored,
            is_content_available,
        }
    }
}

impl ResourceSupplier<Resource<Vec<u8>>> for FileSysResourceSupplier {
    fn resource(&self, uri: &str) -> ResourceSupplied<Resource<Vec<u8>>> {
        let path = Path::new(uri);
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => return ResourceSupplied::NotFound(uri.to_string()),
        };

        if !metadata.is_file() {
            return ResourceSupplied::NotFile(uri.to_string());
        }

        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(_) => return ResourceSupplied::Error(Box::new(IoError::last_os_error())),
        };

        let nature = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if (self.is_resource_ignored)(&file, &nature, &metadata) {
            return ResourceSupplied::Ignored;
        }

        let file_size = metadata.len();
        let created_at = metadata
            .created()
            .ok()
            .map(|systime| DateTime::<Utc>::from(systime));
        let last_modified_at = metadata
            .modified()
            .ok()
            .map(|systime| DateTime::<Utc>::from(systime));
        let content_provider: Option<
            Box<dyn Fn() -> Result<Box<dyn ResourceContent<Vec<u8>>>, Box<dyn Error>>>,
        >;

        if (self.is_content_available)(&file, &nature, &metadata) {
            let uri_clone = uri.to_string(); // Clone for the closure
            content_provider = Some(Box::new(
                move || -> Result<Box<dyn ResourceContent<Vec<u8>>>, Box<dyn Error>> {
                    let mut content = Vec::new();
                    let mut file = fs::File::open(&uri_clone)?;
                    file.read_to_end(&mut content)?;

                    let hash = {
                        let mut hasher = Sha1::new();
                        hasher.update(&content);
                        format!("{:x}", hasher.finalize())
                    };

                    let text = String::from_utf8_lossy(&content).into_owned();

                    Ok(Box::new(BinaryContent {
                        hash,
                        binary: content,
                        text,
                    }) as Box<dyn ResourceContent<Vec<u8>>>)
                },
            ));
        } else {
            content_provider = None;
        }

        ResourceSupplied::Resource(Resource {
            uri: uri.to_string(),
            nature: Some(nature),
            size: Some(file_size),
            created_at,
            last_modified_at,
            content: content_provider,
        })
    }
}

pub struct FileSysUniformResourceSupplier;

impl UniformResourceSupplier<Resource<Vec<u8>>> for FileSysUniformResourceSupplier {
    fn uniform_resource(
        &self,
        resource: Resource<Vec<u8>>,
    ) -> Result<Box<UniformResource<Resource<Vec<u8>>>>, Box<dyn Error>> {
        // Based on the nature of the resource, we determine the type of UniformResource
        if let Some(nature) = &resource.nature {
            match nature.as_str() {
                // Match different file extensions
                "html" => {
                    let html = HTML {
                        resource,
                        head_meta: HashMap::new(),
                    };
                    Ok(Box::new(UniformResource::HTML(html)))
                }
                "json" => {
                    let json = JSON {
                        resource,
                        content: None,
                    };
                    Ok(Box::new(UniformResource::JSON(json)))
                }
                "md" | "mdx" => {
                    let markdown = Markdown {
                        resource,
                        frontmatter: None,
                    };
                    Ok(Box::new(UniformResource::Markdown(markdown)))
                }
                "png" | "gif" | "tiff" | "jpg" | "jpeg" => {
                    let image = Image {
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
    // ignore_paths: RegexSet,    // Now a RegexSet
    // inspect_content: RegexSet, // Now a RegexSet
    resource_supplier: FileSysResourceSupplier,
    uniform_resource_supplier: FileSysUniformResourceSupplier,
}

impl FileSysResourcesWalker {
    pub fn new(
        root_paths: &Vec<String>,
        _ignore_paths: &Vec<regex::Regex>, // Accept Vec<Regex>, but we'll convert it inside
        _inspect_content: &Vec<regex::Regex>, // Accept Vec<Regex>, but we'll convert it inside
    ) -> Result<Self, regex::Error> {
        // Constructor can fail due to RegexSet::new
        // let ignore_set = RegexSet::new(ignore_paths.iter().map(|r| r.as_str()))?;
        // let inspect_set = RegexSet::new(inspect_content.iter().map(|r| r.as_str()))?;
        let resource_supplier = FileSysResourceSupplier::new(
            Box::new(|_file, _nature, _metadata| false), // TODO: use ignore_paths
            Box::new(|_file, _nature, _metadata| false), // TODO: use inspect_content
        );
        let uniform_resource_supplier = FileSysUniformResourceSupplier {};

        Ok(Self {
            root_paths: root_paths.clone(),
            // ignore_paths: ignore_set,
            // inspect_content: inspect_set,
            resource_supplier,
            uniform_resource_supplier,
        })
    }

    pub fn walk_resources<F>(&self, mut encounter_resource: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(UniformResource<Resource<Vec<u8>>>) + 'static,
    {
        for root in &self.root_paths {
            // Walk through each entry in the directory.
            for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                let uri = entry.path().to_string_lossy().into_owned();

                // Use the ResourceSupplier to create a resource from the file.
                match self.resource_supplier.resource(&uri) {
                    ResourceSupplied::Resource(resource) => {
                        // Create a uniform resource for each valid resource.
                        match self.uniform_resource_supplier.uniform_resource(resource) {
                            Ok(uniform_resource) => encounter_resource(*uniform_resource),
                            Err(e) => return Err(e.into()), // Handle error according to your policy
                        }
                    }
                    ResourceSupplied::Error(e) => return Err(e),
                    ResourceSupplied::Ignored => {}
                    ResourceSupplied::NotFile(_) => {}
                    ResourceSupplied::NotFound(_) => {} // TODO: should this be an error?
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
        let result = supplier.resource(test_file_path);

        match result {
            ResourceSupplied::Resource(res) => {
                let content = (res.content.unwrap())().expect("Error obtaining content");
                assert_eq!(content.content_text(Vec::new()), "Hello, world!");
            }
            _ => panic!("Unexpected result from resource()"),
        }

        // Clean up the file system
        fs::remove_file(test_file_path).expect("Unable to delete test file");
    }
}
