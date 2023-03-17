use rocket::figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum EmailSender {
    TestSender,
    Mandrill(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub email_sender: EmailSender,
    pub mandrill_api_host: String,
    pub email_sender_email: String,
    pub email_sender_name: Option<String>,
    pub host: String,
    pub postgres: String,
    pub web_db_pool: u32,
    pub processing_db_pool: u32,
    pub tex_exe: String,
    pub processing_queue_size: usize,
    pub verify_redirects_to: HashMap<String, String>,
    pub notification_email: Option<String>,
    pub email_new_registration_notification_template: String,
    pub email_confirmation_templates: HashMap<String, String>,
    pub email_confirmation_subjects: HashMap<String, String>,
    /// This is rocket level value, not logger one
    pub log_level: rocket::config::LogLevel,
}

impl Config {
    pub fn get() -> Self {
        let figment = Figment::from(rocket::Config::default())
            .merge(Serialized::defaults(rocket::Config::default()))
            .merge(Toml::file("Rocket.toml").nested())
            .merge(Env::prefixed("ROCKET_").global())
            .select(Profile::from_env_or("ROCKET_PROFILE", "default"));

        let mandrill_api_key: Result<String, _> = figment.extract_inner("mandrill_api_key");

        let email_sender = match mandrill_api_key {
            Ok(api_key) => EmailSender::Mandrill(api_key),
            Err(_) => EmailSender::TestSender,
        };

        let mandrill_api_host = figment
            .extract_inner("mandrill_api_host")
            .unwrap_or("https://mandrillapp.com/api/1.0".to_string());

        let email_sender_email = figment
            .extract_inner("email_sender_email")
            .unwrap_or("noreply@localhost".to_string());
        let email_sender_name = figment.extract_inner("email_sender_name").ok();

        let host = figment
            .extract_inner("host")
            .unwrap_or("http://localhost".to_string());

        let postgres = figment
            .extract_inner("postgres")
            .unwrap_or("posthres://orca@localhost/ictunion".to_string());

        let web_db_pool = figment.extract_inner("web_db_pool").unwrap_or(5);
        let processing_db_pool = figment.extract_inner("processing_db_pool").unwrap_or(2);
        let tex_exe = figment
            .extract_inner("tex_exe")
            .unwrap_or("xelatex".to_string());
        let processing_queue_size = figment.extract_inner("processing_queue_size").unwrap_or(16);

        let verify_redirects_to = figment
            .extract_inner("verify_redirects_to")
            .unwrap_or(HashMap::new());

        let notification_email = figment.extract_inner("notification_email").ok();

        let email_new_registration_notification_template = figment
            .extract_inner("email_new_registration_notification_template")
            .unwrap_or("new_registration_notification_template".to_string());

        let email_confirmation_templates = figment
            .extract_inner("email_confirmation_templates")
            .unwrap_or(HashMap::new());

        let email_confirmation_subjects = figment
            .extract_inner("email_confirmation_subjects")
            .unwrap_or(HashMap::new());

        let log_level = figment
            .extract_inner("log_level")
            .unwrap_or(rocket::config::LogLevel::Normal);

        Self {
            email_sender,
            mandrill_api_host,
            email_sender_email,
            email_sender_name,
            host,
            postgres,
            web_db_pool,
            processing_db_pool,
            tex_exe,
            processing_queue_size,
            verify_redirects_to,
            notification_email,
            email_new_registration_notification_template,
            email_confirmation_templates,
            email_confirmation_subjects,
            log_level,
        }
    }

    pub fn verify_redirect_for_local(&self, lang: &str) -> String {
        match self.verify_redirects_to.get(lang) {
            Some(url) => url.to_string(),
            None => self
                .verify_redirects_to
                .get("default")
                .unwrap_or(&self.host)
                .to_string(),
        }
    }

    pub fn email_confirmation_template_for_local(&self, lang: &str) -> String {
        match self.email_confirmation_templates.get(lang) {
            Some(template) => template.to_string(),
            None => self
                .email_confirmation_templates
                .get("default")
                .unwrap_or(&"email_confirmation".to_string())
                .to_string(),
        }
    }

    pub fn email_confirmation_subject_for_local(&self, lang: &str) -> String {
        match self.email_confirmation_subjects.get(lang) {
            Some(sub) => sub.to_string(),
            None => self
                .email_confirmation_subjects
                .get("default")
                .unwrap_or(&"Verify Email Address".to_string())
                .to_string(),
        }
    }
}
