//! Implements email sending via mandrill API
//!
//! This is official curl example for sending email template:
//!
//! ```text
//! curl -X POST \
//!    https://mandrillapp.com/api/1.0/messages/send-template \
//!    -d '{"key":"","template_name":"","template_content":[],"message":{"html":"","text":"","subject":"","from_email":"","from_name":"","to":[],"headers":{},"important":false,"track_opens":false,"track_clicks":false,"auto_text":false,"auto_html":false,"inline_css":false,"url_strip_qs":false,"preserve_recipients":false,"view_content_link":false,"bcc_address":"","tracking_domain":"","signing_domain":"","return_path_domain":"","merge":false,"merge_language":"mailchimp","global_merge_vars":[],"merge_vars":[],"tags":[],"subaccount":"","google_analytics_domains":[],"google_analytics_campaign":"","metadata":{"website":""},"recipient_metadata":[],"attachments":[],"images":[]},"async":false,"ip_pool":"","send_at":""}'
//! ```
//!
//! This module implements RUST wrapper around mandril api.

use hyper::{Body, Client, Method, Request};
use hyper_openssl::HttpsConnector;
use rocket::serde::Serialize;
use rustc_serialize::base64::ToBase64;

use crate::config::{Config, EmailSender};

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Attachement<'a> {
    #[serde(rename = "type")]
    mime_type: &'a str,
    name: &'a str,
    /// Base64 encoded content of file
    content: String,
}

impl<'a> Attachement<'a> {
    pub fn new(name: &'a str, mime_type: &'a str, bytes: &Vec<u8>) -> Self {
        Self {
            name,
            mime_type,
            content: bytes.to_base64(rustc_serialize::base64::MIME),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Recipient {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TemplateMessage<'a> {
    subject: &'a str,
    from_email: &'a str,
    from_name: Option<&'a str>,
    to: Vec<Recipient>,
    tags: Vec<&'a str>,
    attachments: Vec<Attachement<'a>>,
    global_merge_vars: Vec<TemplateContentItem<'a>>,
}

impl<'a> TemplateMessage<'a> {
    pub fn new(subject: &'a str, member: &'a super::MemberDetails, config: &'a Config) -> Self {
        let full_name = format!("{} {}", member.first_name, member.last_name);
        let recipient = Recipient {
            email: member.email.clone(),
            name: full_name,
        };
        Self {
            subject,
            from_email: &config.email_from_email,
            from_name: config.email_from_name.as_deref(),
            to: vec![recipient],
            tags: Vec::new(),
            attachments: Vec::new(),
            global_merge_vars: Vec::new(),
        }
    }

    pub fn attach(&mut self, attachement: Attachement<'a>) -> &mut Self {
        self.attachments.push(attachement);
        self
    }

    pub fn bind(&mut self, item: TemplateContentItem<'a>) -> &mut Self {
        self.global_merge_vars.push(item);
        self
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TemplateContentItem<'a> {
    name: &'a str,
    content: &'a str,
}

impl<'a> TemplateContentItem<'a> {
    pub fn new(name: &'a str, content: &'a str) -> Self {
        Self { name, content }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TemplateEmail<'a> {
    key: &'a str,
    template_name: &'a str,
    template_content: Vec<TemplateContentItem<'a>>,
    message: TemplateMessage<'a>,
}

#[derive(Debug)]
pub struct Sender {
    sender: EmailSender,
    api_base: String,
}

#[derive(Debug)]
pub enum SenderError {
    Http(hyper::http::Error),
    Hyper(hyper::Error),
    Ssl(openssl::error::ErrorStack),
}

impl From<hyper::http::Error> for SenderError {
    fn from(value: hyper::http::Error) -> Self {
        Self::Http(value)
    }
}

impl From<hyper::Error> for SenderError {
    fn from(value: hyper::Error) -> Self {
        Self::Hyper(value)
    }
}

impl From<openssl::error::ErrorStack> for SenderError {
    fn from(value: openssl::error::ErrorStack) -> Self {
        Self::Ssl(value)
    }
}

impl Sender {
    pub fn new(config: &Config) -> Sender {
        Self {
            sender: config.email_sender.clone(),
            api_base: config.mandrill_api_host.clone(),
        }
    }

    pub async fn send_template<'a>(
        &self,
        template_name: &'a str,
        message: TemplateMessage<'a>,
    ) -> Result<(), SenderError> {
        match &self.sender {
            EmailSender::Mandrill(key) => {
                let email = TemplateEmail {
                    key,
                    template_name,
                    template_content: Vec::new(),
                    message,
                };

                self.send_mandrill_email(&email).await?;
                Ok(())
            }
            EmailSender::TestSender => {
                // just print the message
                println!("Email would be sent:");
                println!("\ttemplate: {}", template_name);

                Ok(())
            }
        }
    }

    async fn send_mandrill_email<'a>(&self, email: &TemplateEmail<'a>) -> Result<(), SenderError> {
        let uri = format!("{}/messages/send-template", self.api_base);
        let body = serde_json::to_string(email).expect("convert data to json");
        let req = Request::builder()
            .method(Method::POST)
            .uri(&uri)
            .body(Body::from(body))?;

        let connector = HttpsConnector::new()?;
        let client: Client<_, Body> = Client::builder().build(connector);
        let res = client.request(req).await?;
        println!("result: {:?}", res);

        Ok(())
    }
}
