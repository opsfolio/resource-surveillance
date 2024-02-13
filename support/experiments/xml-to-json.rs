#!/usr/bin/env rust-script
//! This script tests multiple xml to json crates
//! ```cargo
//! [dependencies]
//! serde-xml-rs = "0.6.0"
//! serde_json = { version = "*" }
//! serde = { version = "*" }
//! ```
//! 

use std::fs::File;
use std::io::Write;
use serde_xml_rs::from_str;

fn main() {
    let xml_src = std::fs::read_to_string("../test-fixtures/sample-threat-model-mtm.xml").unwrap();
    let value: serde_json::Value = from_str(&xml_src).unwrap();
    let json_string = serde_json::to_string_pretty(&value).unwrap();

    // Specify the path to the output JSON file
    let path = "serde-xml-rs-output.json";
    let mut file = File::create(path).expect("Failed to create file");

    // Write the pretty JSON string to the file
    file.write_all(json_string.as_bytes()).expect("Failed to write to file");

    println!("JSON output has been written to {}", path);
}
