#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! html_parser = "0.6.3"
//! serde_json = "1.0"
//! serde = { version = "1.0", features = ["derive"] }
//! scraper = "0.19.0"
//! ```

fn main() {
  use scraper::{Html, Selector};
    let html = std::fs::read_to_string("test.html").unwrap();
    let fragment = Html::parse_fragment(&html);
    let selector = Selector::parse("a").unwrap();

    for element in fragment.select(&selector) {
        println!("{element:#?}")
    }
}