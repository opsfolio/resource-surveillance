use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

use crate::capturable::*;
use crate::subprocess::*;

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

#[allow(dead_code)]
#[derive(Debug)]
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

pub trait UriNatureSupplier<Resource> {
    fn uri(&self) -> &String;
    fn nature(&self) -> &Option<String>;
}

impl UriNatureSupplier<ContentResource> for UniformResource<ContentResource> {
    fn uri(&self) -> &String {
        match self {
            UniformResource::CapturableExec(cer) => &cer.executable.uri,
            UniformResource::Html(html) => &html.resource.uri,
            UniformResource::Image(img) => &img.resource.uri,
            UniformResource::Json(json) => &json.resource.uri,
            UniformResource::Markdown(md) => &md.resource.uri,
            UniformResource::PlainText(txt) => &txt.resource.uri,
            UniformResource::SpdxJson(spdx) => &spdx.resource.uri,
            UniformResource::Svg(svg) => &svg.resource.uri,
            UniformResource::Tap(tap) => &tap.resource.uri,
            UniformResource::Toml(toml) => &toml.resource.uri,
            UniformResource::Yaml(yaml) => &yaml.resource.uri,
            UniformResource::Unknown(cr, _alternate) => &cr.uri,
        }
    }

    fn nature(&self) -> &Option<String> {
        match self {
            crate::resource::UniformResource::CapturableExec(cer) => &cer.executable.nature,
            crate::resource::UniformResource::Html(html) => &html.resource.nature,
            crate::resource::UniformResource::Image(img) => &img.resource.nature,
            crate::resource::UniformResource::Json(json) => &json.resource.nature,
            crate::resource::UniformResource::Markdown(md) => &md.resource.nature,
            crate::resource::UniformResource::PlainText(txt) => &txt.resource.nature,
            crate::resource::UniformResource::SpdxJson(spdx) => &spdx.resource.nature,
            crate::resource::UniformResource::Svg(svg) => &svg.resource.nature,
            crate::resource::UniformResource::Tap(tap) => &tap.resource.nature,
            crate::resource::UniformResource::Toml(toml) => &toml.resource.nature,
            crate::resource::UniformResource::Yaml(yaml) => &yaml.resource.nature,
            crate::resource::UniformResource::Unknown(_cr, _alternate) => &None::<String>,
        }
    }
}

pub struct UniformResourceBuilder {
    pub nature_bind: HashMap<String, String>,
}

impl UniformResourceSupplier<ContentResource> for UniformResourceBuilder {
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
        if let Some(supplied_nature) = &resource.nature {
            let mut candidate_nature = supplied_nature.as_str();
            let try_alternate_nature = self.nature_bind.get(candidate_nature);
            if let Some(alternate_bind) = try_alternate_nature {
                candidate_nature = alternate_bind
            }

            match candidate_nature {
                // Match different file extensions
                "html" | "text/html" => {
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
                "json" | "jsonc" | "application/json" => {
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
                "yml" | "application/yaml" => {
                    let yaml = YamlResource {
                        resource,
                        content: None, // TODO parse using serde
                    };
                    Ok(Box::new(UniformResource::Yaml(yaml)))
                }
                "toml" | "application/toml" => {
                    let toml = TomlResource {
                        resource,
                        content: None, // TODO parse using serde
                    };
                    Ok(Box::new(UniformResource::Toml(toml)))
                }
                "md" | "mdx" | "text/markdown" => {
                    let markdown = MarkdownResource { resource };
                    Ok(Box::new(UniformResource::Markdown(markdown)))
                }
                "txt" | "text/plain" => {
                    let plain_text = PlainTextResource { resource };
                    Ok(Box::new(UniformResource::PlainText(plain_text)))
                }
                "png" | "gif" | "tiff" | "jpg" | "jpeg" => {
                    let image = ImageResource {
                        resource,
                        image_meta: HashMap::new(), // TODO add meta data, infer type from content
                    };
                    Ok(Box::new(UniformResource::Image(image)))
                }
                "svg" | "image/svg+xml" => {
                    let svg = SvgResource { resource };
                    Ok(Box::new(UniformResource::Svg(svg)))
                }
                "tap" => {
                    let tap = TestAnythingResource { resource };
                    Ok(Box::new(UniformResource::Tap(tap)))
                }
                _ => Ok(Box::new(UniformResource::Unknown(
                    resource,
                    try_alternate_nature.cloned(),
                ))),
            }
        } else {
            Err(format!(
                "Unable to obtain nature for {} from supplied resource",
                resource.uri
            )
            .into())
        }
    }
}
