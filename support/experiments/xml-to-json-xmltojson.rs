#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! xmltojson = "0.1.3"
//! serde_json = { version = "*" }
//! serde = { version = "*" }
//! ```
//! 

use std::fs::File;
use std::io::Write;
use xmltojson::to_json;

fn main() {
    let xml_src = std::fs::read_to_string("../test-fixtures/sample-threat-model-mtm.xml").unwrap();
    let value: serde_json::Value = to_json(&xml_src).unwrap();
    let json_string = serde_json::to_string_pretty(&value).unwrap();

    // Specify the path to the output JSON file
    let path = "output.json";
    let mut file = File::create(path).expect("Failed to create file");

    // Write the pretty JSON string to the file
    file.write_all(json_string.as_bytes()).expect("Failed to write to file");

    println!("JSON output has been written to {}", path);
}
