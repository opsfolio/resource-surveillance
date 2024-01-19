use std::sync::Arc;

use pgwire::api::MakeHandler;

use crate::{parser::UdiPgpQueryParser, config::UdiPgpConfig};

pub mod query_handler;

//TODO: connect the suppliers here
#[derive(Debug, Clone)]
pub struct UdiPgpProcessor {
    query_parser: UdiPgpQueryParser,
    config: UdiPgpConfig
}

impl UdiPgpProcessor {
    pub fn new(config: &UdiPgpConfig) -> Self {
        UdiPgpProcessor {
            query_parser: UdiPgpQueryParser::new(),
            config: config.clone()
        }
    }
}

impl MakeHandler for UdiPgpProcessor {
    type Handler = Arc<UdiPgpProcessor>;

    fn make(&self) -> Self::Handler {
        Arc::new(UdiPgpProcessor {
            query_parser: self.query_parser.clone(),
            config: self.config.clone()
        })
    }
}
