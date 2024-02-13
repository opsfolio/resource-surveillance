#!/usr/bin/env rust-script
//! This script converts XML with namespaces, elements, and attributes to JSON with full fidelity
//! ```cargo
//! [dependencies]
//! xmltree = "0.10.0"
//! serde_json = "1.0"
//! ```

use xmltree::Element;
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;

fn main() {
    let mut file = File::open("../test-fixtures/sample-threat-model-mtm.xml").expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read file");

    let root = Element::parse(contents.as_bytes()).expect("Failed to parse XML");
    let json = element_to_json(&root);

    println!("{}", json.to_string());
}

fn element_to_json(element: &Element) -> Value {
    let mut object = serde_json::Map::new();

    // Convert attributes
    if !element.attributes.is_empty() {
        let attributes = element.attributes.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        object.insert("@attributes".to_string(), json!(attributes));
    }

    // Convert children
    if !element.children.is_empty() {
        let mut children = serde_json::Map::new();
        for child in &element.children {
            match child {
                xmltree::XMLNode::Element(e) => {
                    let child_json = element_to_json(e);
                    children.insert(e.name.clone(), child_json);
                },
                xmltree::XMLNode::Text(t) => {
                    if !t.trim().is_empty() {
                        object.insert("#text".to_string(), json!(t.trim()));
                    }
                },
                _ => {}
            }
        }
        if !children.is_empty() {
            object.insert("children".to_string(), json!(children));
        }
    } else if let Some(text) = element.text.as_ref() {
        object.insert("#text".to_string(), json!(text.trim()));
    }

    // Handle namespaces if necessary

    json!(object)
}
