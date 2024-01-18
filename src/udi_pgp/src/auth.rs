use serde::{Deserialize, Serialize};

/// Authentication that gets passed to pgwire
// TODO think of making it base64
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Auth {
    username: String,
    password: String,
}

impl Auth {
    pub fn new(u: &str, p: &str) -> Self {
        Auth {
            username: u.to_string(),
            password: p.to_string(),
        }
    }

    pub fn user(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}
