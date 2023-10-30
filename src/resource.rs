use std::error::Error;

use chrono::{DateTime, Utc};

pub trait ResourceBinaryContent {
    fn content_digest_hash(&self) -> &str;
    fn content_binary(&self) -> &Vec<u8>;
}

pub trait ResourceTextContent {
    fn content_digest_hash(&self) -> &str;
    fn content_text(&self) -> &str;
}

pub struct Resource {
    pub uri: String,
    pub nature: Option<String>,
    pub size: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub content_binary_supplier:
        Option<Box<dyn Fn() -> Result<Box<dyn ResourceBinaryContent>, Box<dyn Error>>>>,
    pub content_text_supplier:
        Option<Box<dyn Fn() -> Result<Box<dyn ResourceTextContent>, Box<dyn Error>>>>,
}

pub enum ResourceSupplied<T> {
    Ignored(String),
    NotFound(String),
    NotFile(String),
    Resource(T),
    Error(Box<dyn Error>),
}

pub trait ResourceSupplier<Resource> {
    fn resource(&self, uri: &str) -> ResourceSupplied<Resource>;
}
