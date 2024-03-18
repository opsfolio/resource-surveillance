#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! imap = "2.4.1"
//! native-tls = "0.2"
//! serde_json = "1.0"
//! serde = { version = "1.0", features = ["derive"] }
//! mailparse = "0.13"
//! base64 = "0.21.7"
//! ```

extern crate imap;
extern crate mailparse;
extern crate native_tls;
extern crate serde;
extern crate serde_json;
extern crate base64;

use mailparse::{parse_mail, MailHeaderMap};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
struct GmailOAuth2 {
  user: String,
  access_token: String,
}

impl imap::Authenticator for GmailOAuth2 {
type Response = String;
#[allow(unused_variables)]
fn process(&self, data: &[u8]) -> Self::Response {
    format!(
        "user={}\x01auth=Bearer {}\x01\x01",
        self.user, self.access_token
    )
}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  
  #[derive(Serialize, Deserialize, Debug)]
  struct Email {
      subject: String,
      from: String,
      cc: String,
      bcc: String,
      references: String,
      in_reply_to: String,
      message_id: String,
      to: String,
      date: String,
      text_plain: Vec<String>,
      text_html: Vec<String>,
  }


    let gmail_auth = GmailOAuth2 {
        user: String::from("baasisek01@gmail.com"),
        access_token: String::from("<access_token>"),
    };

    let client = imap::ClientBuilder::new("imap.gmail.com", 993)
        .connect()
        .expect("Could not connect to imap.gmail.com");
    
    // Login
    let mut imap_session = client.login(username, password).map_err(|e| e.0)?;
    
    let mailbox = imap_session.select("INBOX")?;

    // Check the number of messages in the INBOX
    let messages_total = mailbox.exists;
    // println!("{:#?}", messages_total);
    // select the last 5 emails
    let start = if messages_total >= 5 { messages_total - 4 } else { 1 };
    let fetch_range = format!("{}:*", start);
    
    // Fetch messages in the INBOX
    let messages = imap_session.fetch(fetch_range, "RFC822")?;

    let mut emails = Vec::new();

    for message in messages.iter() {
      let body = message.body().expect("message did not have a body!");
      let body_str = std::str::from_utf8(body)
        .expect("message was not valid utf-8")
        .to_string();    

        let parsed_mail = parse_mail(body_str.as_bytes())?;

          let email = Email {
            subject: parsed_mail.headers.get_first_value("Subject").unwrap_or_default(),
            from: parsed_mail.headers.get_first_value("From").unwrap_or_default(),
            to: parsed_mail.headers.get_first_value("To").unwrap_or_default(),
            cc: parsed_mail.headers.get_first_value("Cc").unwrap_or_default(),
            bcc: parsed_mail.headers.get_first_value("Bcc").unwrap_or_default(),
            references: parsed_mail.headers.get_first_value("References").unwrap_or_default(),
            in_reply_to: parsed_mail.headers.get_first_value("In-Reply-To").unwrap_or_default(),
            message_id: parsed_mail.headers.get_first_value("Message-ID").unwrap_or_default(),
            date: parsed_mail.headers.get_first_value("Date").unwrap_or_default(),
            text_plain: parsed_mail.subparts.iter().filter_map(|p| {
                if p.ctype.mimetype == "text/plain" {
                    p.get_body().ok()
                } else {
                    None
                }
            }).collect(),
            text_html: parsed_mail.subparts.iter().filter_map(|p| {
                if p.ctype.mimetype == "text/html" {
                    p.get_body().ok()
                } else {
                    None
                }
            }).collect(),
        };
        emails.push(email);

      }

    // Log out
    imap_session.logout()?;

    let emails = serde_json::to_string_pretty(&emails)?;
    println!("{}", emails);

    Ok(())
}

