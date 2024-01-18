use crate::{CapturableExecResource, HtmlResource, ImageResource, JsonResource, JsonableTextResource, MarkdownResource, PlainTextResource, SourceCodeResource, XmlResource, ContentResource};

impl UniformResource<ContentResource> {
    pub fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        match self {
            UniformResource::CapturableExec(capturable) => capturable.insert(urw_state, entry),
            UniformResource::Html(html) => html.insert(urw_state, entry),
            UniformResource::Json(json) => json.insert(urw_state, entry),
            UniformResource::JsonableText(jtr) => jtr.insert(urw_state, entry),
            UniformResource::Image(img) => img.insert(urw_state, entry),
            UniformResource::Markdown(md) => md.insert(urw_state, entry),
            UniformResource::PlainText(txt) => txt.insert(urw_state, entry),
            UniformResource::SourceCode(sc) => sc.insert(urw_state, entry),
            UniformResource::Xml(xml) => xml.insert(urw_state, entry),
            UniformResource::Unknown(unknown, tried_alternate_nature) => {
                if let Some(tried_alternate_nature) = tried_alternate_nature {
                    entry.tried_alternate_nature = Some(tried_alternate_nature.clone());
                }
                unknown.insert(urw_state, entry)
            }
        }
    }
}