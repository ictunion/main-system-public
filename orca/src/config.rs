use figment::{
    Figment, Profile,
    providers::{Env, Format, Toml},
};
use serde::de::{self, DeserializeOwned, Unexpected};
use serde::{Deserialize, Deserializer};

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

use self::templates::Templates;
pub mod templates;

#[derive(Debug, Clone)]
pub struct Config {
    pub email_sender_email: String,
    pub email_sender_name: Option<String>,
    pub treasurer_reply_email: String,
    pub treasurer_reply_name: String,
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
    pub log_level: LogLevel,
    /// Address the HTTP server binds to
    pub address: IpAddr,
    /// Port the HTTP server listens on
    pub port: u16,
    /// Maximum size of a JSON request body in bytes (`limits.json`)
    pub json_limit: usize,
    pub smtp_host: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub keycloak_host: Option<String>,
    pub keycloak_realm: Option<String>,
    pub keycloak_client_id: Option<String>,
    pub keycloak_members_group_id: Option<Uuid>,
    pub templates: Templates<'static>,
    pub listmonk_password: Option<String>,
    pub listmonk_username: Option<String>,
    pub listmonk_host: Option<String>,
    /// Shared secret bearer token for machine-to-machine bank-system sync
    pub bank_sync_token: Option<String>,
}

impl Config {
    /// Read configuration
    ///
    /// # Panics
    ///
    /// If one or more required configuration fields aren't present or a value is invalid. It's pointless to continue execution.
    #[must_use]
    pub fn get() -> Self {
        let figment = Figment::new()
            // TODO: rename Rocket.toml to Orca.toml later, when we are sure that everything works
            .merge(Toml::file("Rocket.toml").nested())
            // TODO: remove ROCKET_ prefix
            .merge(Env::prefixed("ROCKET_").split("__").global())
            .select(Profile::from_env_or("ROCKET_PROFILE", "default"));

        let email_sender_email = figment
            .extract_inner("email_sender_email")
            .unwrap_or("noreply@localhost".to_string());
        let email_sender_name = figment.extract_inner("email_sender_name").ok();
        let treasurer_reply_email = figment
            .extract_inner("treasurer_reply_email")
            .unwrap_or("treasurer@ictunion.cz".to_string());
        let treasurer_reply_name = figment
            .extract_inner("treasurer_reply_name")
            .unwrap_or("ICT Union Treasurer".to_string());

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

        let email_confirmation_subjects: HashMap<String, String> = figment
            .extract_inner("email_confirmation_subjects")
            .unwrap_or_default();

        let log_level = extract_or(&figment, "log_level", LogLevel::build_default());
        let address = extract_or(&figment, "address", IpAddr::V4(Ipv4Addr::LOCALHOST));
        let port = extract_or(&figment, "port", 8000);
        let json_limit = extract_or(&figment, "limits.json", 1_048_576);

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

        let keycloak_members_group_id: Option<Uuid> =
            figment.extract_inner("keycloak_members_group_id").ok();

        let listmonk_password: Option<String> = figment.extract_inner("listmonk_password").ok();

        let listmonk_username: Option<String> = figment.extract_inner("listmonk_username").ok();

        let listmonk_host: Option<String> = figment.extract_inner("listmonk_host").ok();

        let bank_sync_token: Option<String> = figment.extract_inner("bank_sync_token").ok();

        Self {
            email_sender_email,
            email_sender_name,
            treasurer_reply_email,
            treasurer_reply_name,
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
            address,
            port,
            json_limit,
            smtp_host,
            smtp_user,
            smtp_password,
            keycloak_host,
            keycloak_realm,
            keycloak_client_id,
            keycloak_members_group_id,
            templates,
            listmonk_password,
            listmonk_username,
            listmonk_host,
            bank_sync_token,
        }
    }

    #[must_use]
    pub fn verify_redirect_for_local(&self, lang: &str) -> String {
        match self.verify_redirects_to.get(lang) {
            Some(url) => url.clone(),
            None => self
                .verify_redirects_to
                .get("default")
                .unwrap_or(&self.host)
                .clone(),
        }
    }

    #[must_use]
    pub fn email_confirmation_subject_for_local(&self, lang: &str) -> String {
        match self.email_confirmation_subjects.get(lang) {
            Some(sub) => sub.clone(),
            None => self
                .email_confirmation_subjects
                .get("default")
                .unwrap_or(&"Verify Email Address".to_string())
                .clone(),
        }
    }
}

/// Like `extract_inner`, but only a missing key falls back to `default`.
/// A present but invalid value stops the startup.
fn extract_or<T: DeserializeOwned>(figment: &Figment, key: &str, default: T) -> T {
    match figment.extract_inner(key) {
        Ok(value) => value,
        Err(err) if err.missing() => default,
        Err(err) => panic!("Invalid `{key}` configuration: {err}"),
    }
}

/// Log level with the same values and meaning as Rocket's `log_level` setting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Critical,
    Normal,
    Debug,
    Off,
}

impl LogLevel {
    /// `normal` in debug builds, `critical` in release builds (same as Rocket)
    fn build_default() -> Self {
        if cfg!(debug_assertions) {
            Self::Normal
        } else {
            Self::Critical
        }
    }
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Critical => Self::Warn,
            LogLevel::Normal => Self::Info,
            LogLevel::Debug => Self::Trace,
            LogLevel::Off => Self::Off,
        }
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.to_ascii_lowercase().as_str() {
            "critical" => Ok(Self::Critical),
            "normal" => Ok(Self::Normal),
            "debug" => Ok(Self::Debug),
            "off" => Ok(Self::Off),
            _ => Err(de::Error::invalid_value(
                Unexpected::Str(&value),
                &"one of `critical`, `normal`, `debug` or `off`",
            )),
        }
    }
}
