use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

use crate::capturable::*;

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

pub type BinaryExecOutput = (
    Box<dyn BinaryContent>,
    subprocess::ExitStatus,
    Option<String>,
);
pub type BinaryExecOutputSupplier =
    Box<dyn Fn(Option<String>) -> Result<BinaryExecOutput, Box<dyn Error>>>;

pub type TextExecOutput = (Box<dyn TextContent>, subprocess::ExitStatus, Option<String>);
pub type TextExecOutputSupplier =
    Box<dyn Fn(Option<String>) -> Result<TextExecOutput, Box<dyn Error>>>;

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

pub enum ContentResourceSupplied<T> {
    Ignored(String),
    NotFound(String),
    NotFile(String),
    Resource(T),
    Error(Box<dyn Error>),
}

pub trait ContentResourceSupplier<Resource> {
    fn content_resource(&self, uri: &str) -> ContentResourceSupplied<Resource>;
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
