use std::error::Error;

use sha1::{Digest, Sha1};
use tokio::runtime::Runtime;

use crate::capturable::*;
use crate::frontmatter::frontmatter;
use crate::resource::*;
use crate::shell::*;
use crate::subprocess::*;

const DENO_TASK_SHELL_NOTEBOOK_KERNEL_ID: &str = "DenoTaskShell";

pub struct ExecutedCellTextContent {
    pub hash: String,
    pub text: String,
}

impl TextContent for ExecutedCellTextContent {
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

#[derive(Debug, Clone)]
pub struct NotebookCellCode {
    pub notebook_name: String,
    pub notebook_kernel_id: String,
    pub cell_name: String,
    pub interpretable_code: String,
}

#[derive(Debug, Clone)]
pub enum NotebookCell {
    NotRustExecutable(NotebookCellCode),
    ExecutableSqlInJsonOut(NotebookCellCode),
    ExecutableDenoTaskShellInJsonOut(NotebookCellCode),
}

impl NotebookCell {
    pub fn from(nbcc: &NotebookCellCode) -> NotebookCell {
        if nbcc.notebook_kernel_id == DENO_TASK_SHELL_NOTEBOOK_KERNEL_ID {
            NotebookCell::ExecutableDenoTaskShellInJsonOut(nbcc.clone())
        } else {
            NotebookCell::NotRustExecutable(nbcc.clone())
        }
    }

    pub fn content_resource(&self) -> ContentResourceSupplied<ContentResource> {
        let nbcc = match self {
            NotebookCell::NotRustExecutable(nbcc) => nbcc,
            NotebookCell::ExecutableSqlInJsonOut(nbcc) => nbcc,
            NotebookCell::ExecutableDenoTaskShellInJsonOut(nbcc) => nbcc,
        };

        let file_size = nbcc.interpretable_code.len();
        let created_at = Some(chrono::offset::Utc::now());
        let last_modified_at = Some(chrono::offset::Utc::now());
        let content_binary_supplier: Option<BinaryContentSupplier> = None;
        let content_text_supplier: Option<TextContentSupplier>;
        let capturable_executable: Option<CapturableExecutable> = None;
        let capturable_exec_binary_supplier: Option<BinaryExecOutputSupplier> = None;
        let capturable_exec_text_supplier: Option<TextExecOutputSupplier> = None;

        let closure_self = self.clone();
        content_text_supplier = Some(Box::new(
            move || -> Result<Box<dyn TextContent>, Box<dyn Error>> {
                let mut text = String::new();

                match &closure_self {
                    NotebookCell::NotRustExecutable(_) => {}
                    NotebookCell::ExecutableSqlInJsonOut(_) => {
                        text.push_str("[NotebookCell::ExecutableSql] Not implemented yet")
                    }
                    NotebookCell::ExecutableDenoTaskShellInJsonOut(nbcc) => {
                        let mut shell_result_supplier = ShellResultSupplier::new(None);
                        // TODO: make this a OnceCell const
                        let runtime = Runtime::new().unwrap(); // Create a new Tokio runtime
                        let result = shell_result_supplier.result(
                            &runtime,
                            &nbcc.interpretable_code,
                            vec![],
                        );
                        let stdout_json = result.stdout_json_text(None);
                        text.push_str(&stdout_json)
                    }
                };

                let hash = {
                    let mut hasher = Sha1::new();
                    hasher.update(&text);
                    format!("{:x}", hasher.finalize())
                };

                Ok(Box::new(ExecutedCellTextContent { hash, text }) as Box<dyn TextContent>)
            },
        ));

        ContentResourceSupplied::Resource(ContentResource {
            uri: format!("notebook://{}/{}", nbcc.notebook_name, nbcc.cell_name),
            nature: Some("json".to_string()),
            size: Some(file_size as u64),
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
