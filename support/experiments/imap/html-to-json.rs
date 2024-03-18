#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! html_parser = "0.6.3"
//! serde_json = "1.0"
//! ammonia = "3.3.0"
//! serde = { version = "1.0", features = ["derive"] }
//! ```

fn main() {
    let html = std::fs::read_to_string("test.html").unwrap();
    //     println!("{html}");
    let html = ammonia::clean(&html);
    // println!("{html}");
    let parsed_html = html_parser::Dom::parse(&html).unwrap();
    let html_json = parsed_html.to_json_pretty().unwrap();
    println!("{html_json:#?}");
}