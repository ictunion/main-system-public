use rocket::figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};

use std::collections::HashMap;

use self::templates::Templates;
pub mod templates;

#[derive(Debug, Clone)]
pub struct Config {
    pub email_sender_email: String,
    pub email_sender_name: Option<String>,
    pub host: String,
    pub admin_host: Option<String>,
    pub postgres: String,
    pub web_db_pool: u32,
    pub processing_db_pool: u32,
    pub tex_exe: String,
    pub processing_queue_size: usize,
    pub verify_redirects_to: HashMap<String, String>,
    pub notification_email: Option<String>,
    pub email_confirmation_subjects: HashMap<String, String>,
    /// This is rocket level value, not logger one
    pub log_level: rocket::config::LogLevel,
    pub smtp_host: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub keycloak_host: Option<String>,
    pub keycloak_realm: Option<String>,
    pub keycloak_client_id: Option<String>,
    pub templates: Templates<'static>,
}

impl Config {
    pub fn get() -> Self {
        let figment = Figment::from(rocket::Config::default())
            .merge(Serialized::defaults(rocket::Config::default()))
            .merge(Toml::file("Rocket.toml").nested())
            .merge(Env::prefixed("ROCKET_").global())
            .select(Profile::from_env_or("ROCKET_PROFILE", "default"));

        let email_sender_email = figment
            .extract_inner("email_sender_email")
            .unwrap_or("noreply@localhost".to_string());
        let email_sender_name = figment.extract_inner("email_sender_name").ok();

        let host = figment
            .extract_inner("host")
            .unwrap_or("http://localhost".to_string());

        let admin_host = figment.extract_inner("admin_host").ok();

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
            .unwrap_or_default();

        let notification_email = figment.extract_inner("notification_email").ok();

        let email_confirmation_subjects = figment
            .extract_inner("email_confirmation_subjects")
            .unwrap_or_default();

        let log_level = figment
            .extract_inner("log_level")
            .unwrap_or(rocket::config::LogLevel::Normal);

        let smtp_host = figment
            .extract_inner("smtp_host")
            .expect("smtp_host must be configured");

        let smtp_user = figment
            .extract_inner("smtp_user")
            .expect("smtp_user must be configured");

        let smtp_password = figment
            .extract_inner("smtp_password")
            .expect("smtp_password must be configured");

        let templates_path: String = figment.extract_inner("templates_dir").unwrap();

        let mut templates = Templates::new();
        // This makes app fail at startup in case there is some error which we want
        templates.preload_templates(&templates_path).unwrap();

        // read keycloak settings
        let keycloak_host: Option<String> = figment.extract_inner("keycloak_host").ok();

        let keycloak_realm: Option<String> = figment.extract_inner("keycloak_realm").ok();

        let keycloak_client_id: Option<String> = figment.extract_inner("keycloak_client_id").ok();

        Self {
            email_sender_email,
            email_sender_name,
            host,
            admin_host,
            postgres,
            web_db_pool,
            processing_db_pool,
            tex_exe,
            processing_queue_size,
            verify_redirects_to,
            notification_email,
            email_confirmation_subjects,
            log_level,
            smtp_host,
            smtp_user,
            smtp_password,
            keycloak_host,
            keycloak_realm,
            keycloak_client_id,
            templates,
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
