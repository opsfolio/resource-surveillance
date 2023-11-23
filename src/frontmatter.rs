use regex::Regex;
use serde_json::Value as JsonValue;
use std::error::Error;

pub enum FrontmatterNature {
    None,
    YamlFM,
    TomlFM,
    JsonFM,
}

pub type FrontmatterComponents = (
    crate::frontmatter::FrontmatterNature,
    Option<String>,
    Result<JsonValue, Box<dyn Error>>,
    String,
);

pub fn frontmatter(text: &str) -> FrontmatterComponents {
    // Define regex patterns for YAML, TOML, and JSON frontmatter
    // The ending delimiter must be alone on its line and followed by a newline;
    // - `[\s\S]` is a character class that matches any whitespace character (\s)
    //    and any non-whitespace character (\S), which effectively matches any
    //    character, including newlines.
    // - `*?` is a non-greedy quantifier that matches as few characters as possible
    //    to satisfy the pattern.
    let yaml_regex = Regex::new(r"^---\n([\s\S]*?)\n---\n").unwrap();
    let toml_regex = Regex::new(r"^\+\+\+\n([\s\S]*?)\n\+\+\+\n").unwrap();
    let json_regex = Regex::new(r"^\{\n([\s\S]*?)\n\}\n").unwrap();

    let nature: FrontmatterNature;
    let content;
    let mut frontmatter_raw = None;
    let mut frontmatter_json: Result<JsonValue, Box<dyn Error>> =
        Err("No frontmatter found".into());

    // Check for YAML frontmatter
    if let Some(caps) = yaml_regex.captures(text) {
        let fm = caps.get(1).unwrap().as_str();
        frontmatter_raw = Some(caps.get(0).unwrap().as_str().to_string());
        content = yaml_regex.replace(text, "").to_string();
        frontmatter_json = serde_yaml::from_str(fm).map_err(Into::into);
        nature = FrontmatterNature::YamlFM;
    }
    // Check for TOML frontmatter
    else if let Some(caps) = toml_regex.captures(text) {
        let fm = caps.get(1).unwrap().as_str();
        frontmatter_raw = Some(caps.get(0).unwrap().as_str().to_string());
        content = toml_regex.replace(text, "").to_string();
        frontmatter_json = toml::from_str(fm).map_err(Into::into);
        nature = FrontmatterNature::TomlFM;
    }
    // Check for JSON frontmatter
    else if let Some(caps) = json_regex.captures(text) {
        let fm = caps.get(0).unwrap().as_str();
        frontmatter_raw = Some(caps.get(0).unwrap().as_str().to_string());
        content = json_regex.replace(text, "").to_string();
        frontmatter_json = serde_json::from_str(fm).map_err(Into::into);
        nature = FrontmatterNature::JsonFM;
    }
    // If no frontmatter is found, the content remains unchanged
    else {
        content = text.to_string();
        nature = FrontmatterNature::None;
    }

    (nature, frontmatter_raw, frontmatter_json, content)
}

// The rest of the code, including tests, remains the same.

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_yaml_frontmatter() {
        let text = "---\ntitle: Example\n---\nContent goes here.";
        let (nature, fm, fm_json, content) = frontmatter(text);
        assert!(matches!(nature, FrontmatterNature::YamlFM));
        assert_eq!(fm, Some("---\ntitle: Example\n---\n".to_string()));
        assert!(fm_json.is_ok());
        assert_eq!(fm_json.unwrap(), json!({"title": "Example"}));
        assert_eq!(content, "Content goes here.");
    }

    #[test]
    fn test_toml_frontmatter() {
        let text = "+++\ntitle = \"Example\"\n+++\nContent goes here.";
        let (nature, fm, fm_json, content) = frontmatter(text);
        assert!(matches!(nature, FrontmatterNature::TomlFM));
        assert_eq!(fm, Some("+++\ntitle = \"Example\"\n+++\n".to_string()));
        assert!(fm_json.is_ok());
        assert_eq!(fm_json.unwrap(), json!({"title": "Example"}));
        assert_eq!(content, "Content goes here.");
    }

    #[test]
    fn test_json_frontmatter() {
        let text = "{\n\"title\": \"Example\"\n}\nContent goes here.";
        let (nature, fm, fm_json, content) = frontmatter(text);
        assert!(matches!(nature, FrontmatterNature::JsonFM));
        assert_eq!(fm, Some("{\n\"title\": \"Example\"\n}\n".to_string()));
        assert!(fm_json.is_ok());
        assert_eq!(fm_json.unwrap(), json!({"title": "Example"}));
        assert_eq!(content, "Content goes here.");
    }

    #[test]
    fn test_no_frontmatter() {
        let text = "Content goes here.";
        let (nature, fm, fm_json, content) = frontmatter(text);
        assert!(matches!(nature, FrontmatterNature::None));
        assert_eq!(fm, None);
        assert!(fm_json.is_err());
        assert_eq!(content, "Content goes here.");
    }

    #[test]
    fn test_invalid_yaml_frontmatter() {
        let text = "---\ntitle: Example\n----\nContent goes here."; // Invalid YAML frontmatter
        let (_, _, fm_json, _) = frontmatter(text);
        assert!(fm_json.is_err());
    }

    #[test]
    fn test_invalid_toml_frontmatter() {
        let text = "+++\ntitle = 'Example'\n++++\nContent goes here."; // Invalid TOML frontmatter
        let (_, _, fm_json, _) = frontmatter(text);
        assert!(fm_json.is_err());
    }

    #[test]
    fn test_invalid_json_frontmatter() {
        let text = "{\n\"title\": \"Example\"\n\nContent goes here."; // Missing closing }
        let (_, _, fm_json, _) = frontmatter(text);
        assert!(fm_json.is_err());
    }
}
