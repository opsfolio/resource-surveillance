use std::sync::Arc;

use pgwire::api::MakeHandler;

use crate::parser::UdiPgpQueryParser;

pub mod query_handler;

#[derive(Debug, Clone)]
pub struct UdiPgpProcessor {
    query_parser: UdiPgpQueryParser,
}

impl UdiPgpProcessor {
    pub fn new() -> Self {
        UdiPgpProcessor {
            query_parser: UdiPgpQueryParser::new(),
        }
    }
}

impl MakeHandler for UdiPgpProcessor {
    type Handler = Arc<UdiPgpProcessor>;

    fn make(&self) -> Self::Handler {
        Arc::new(UdiPgpProcessor {
            query_parser: UdiPgpQueryParser::new(),
        })
    }
}
