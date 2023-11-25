use std::error::Error;
use std::fs;
use std::io::{Error as IoError, Read, Write};
use std::path::Path;

use chrono::{DateTime, Utc};
use sha1::{Digest, Sha1};

use crate::frontmatter::frontmatter;
use crate::resource::*;

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

pub type FileSysPathQualifier = Box<dyn Fn(&Path, &str, &fs::File) -> bool>;
pub type FileSysPathCapExecQualifier =
    Box<dyn Fn(&Path, &str, &fs::File) -> Option<CapturableExecutable>>;

pub enum FileSysPathOption {
    No,
    Yes,
    Check(FileSysPathQualifier),
}

pub struct FileSysPathContentOptions {
    pub is_ignored: FileSysPathOption,
    pub has_content: FileSysPathOption,
    pub is_capturable_executable: Option<FileSysPathCapExecQualifier>,
}

pub fn fs_path_content_resource(
    uri: &str,
    options: &FileSysPathContentOptions,
) -> ContentResourceSupplied<ContentResource> {
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

    match &options.is_ignored {
        FileSysPathOption::No => {}
        FileSysPathOption::Yes => return ContentResourceSupplied::Ignored(uri.to_string()),
        FileSysPathOption::Check(is_ignored) => {
            if (is_ignored)(path, &nature, &file) {
                return ContentResourceSupplied::Ignored(uri.to_string());
            }
        }
    }

    let file_size = metadata.len();
    let created_at = metadata.created().ok().map(DateTime::<Utc>::from);
    let last_modified_at = metadata.modified().ok().map(DateTime::<Utc>::from);
    let content_binary_supplier: Option<BinaryContentSupplier>;
    let content_text_supplier: Option<TextContentSupplier>;
    let capturable_executable: Option<CapturableExecutable>;
    let capturable_exec_binary_supplier: Option<BinaryExecOutputSupplier>;
    let capturable_exec_text_supplier: Option<TextExecOutputSupplier>;

    if options.is_capturable_executable.is_some() {
        if let Some(capturable) =
            (options.is_capturable_executable.as_ref().unwrap())(path, &nature, &file)
        {
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
                            .stdout(subprocess::Redirection::Pipe)
                            .stderr(subprocess::Redirection::Pipe);

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
    } else {
        capturable_executable = None;
        capturable_exec_binary_supplier = None;
        capturable_exec_text_supplier = None;
    }

    let acquire_content = match &options.has_content {
        FileSysPathOption::No => false,
        FileSysPathOption::Yes => true,
        FileSysPathOption::Check(has_content) => (has_content)(path, &nature, &file),
    };

    if acquire_content {
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

// For the unit test, we use the built-in testing framework
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_fs_path_content_resource() {
        let fspc_options = FileSysPathContentOptions {
            is_ignored: FileSysPathOption::No,
            has_content: FileSysPathOption::Yes,
            is_capturable_executable: None,
        };

        // Create a file for the test environment, writing some content
        let test_file_path = "test.txt";
        let test_data = b"Hello, world!";
        fs::write(test_file_path, test_data).expect("Unable to write test file");

        // Use the supplier to get a resource
        let result = fs_path_content_resource(test_file_path, &fspc_options);

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
