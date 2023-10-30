use std::collections::HashMap;
use std::error::Error;

use serde_json::Value as JsonValue;

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
