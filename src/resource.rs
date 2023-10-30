use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

pub trait BinaryContent {
    fn content_digest_hash(&self) -> &str;
    fn content_binary(&self) -> &Vec<u8>;
}

pub trait TextContent {
    fn content_digest_hash(&self) -> &str;
    fn content_text(&self) -> &str;
}

pub struct ContentResource {
    pub uri: String,
    pub nature: Option<String>,
    pub size: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub content_binary_supplier:
        Option<Box<dyn Fn() -> Result<Box<dyn BinaryContent>, Box<dyn Error>>>>,
    pub content_text_supplier:
        Option<Box<dyn Fn() -> Result<Box<dyn TextContent>, Box<dyn Error>>>>,
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

pub struct HTML<Resource> {
    pub resource: Resource,
    pub head_meta: HashMap<String, String>,
}

pub struct Image<Resource> {
    pub resource: Resource,
    pub image_meta: HashMap<String, String>,
}

pub struct JSON<Resource> {
    pub resource: Resource,
    pub content: Option<JsonValue>, // The actual JSON content
}

pub struct Markdown<Resource> {
    pub resource: Resource,
    pub frontmatter: Option<JsonValue>,
}

pub enum UniformResource<Resource> {
    Unknown(Resource),
    Image(Image<Resource>),
    Markdown(Markdown<Resource>),
    JSON(JSON<Resource>),
    HTML(HTML<Resource>),
}

pub trait UniformResourceSupplier<Resource> {
    fn uniform_resource(
        &self,
        rs: Resource,
    ) -> Result<Box<UniformResource<Resource>>, Box<dyn Error>>;
}
