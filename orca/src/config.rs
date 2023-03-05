use rocket::figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};

#[derive(Debug, PartialEq, Clone)]
pub enum EmailSender {
    TestSender,
    Mandrill(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub email_sender: EmailSender,
    pub mandrill_api_host: String,
    pub email_from_email: String,
    pub email_from_name: Option<String>,
    pub host: String,
    pub email_confirmation_template_name: String,
    pub postgres: String,
    pub web_db_pool: u32,
    pub processing_db_pool: u32,
    pub tex_exe: String,
    pub processing_queue_size: usize,
    pub verify_redirects_to: String,
    pub notification_email: Option<String>,
    pub new_member_notification_template: String,
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

        let email_from_email = figment
            .extract_inner("email_from_email")
            .unwrap_or("noreply@localhost".to_string());
        let email_from_name = figment.extract_inner("email_from_name").ok();

        let host = figment
            .extract_inner("host")
            .unwrap_or("http://localhost".to_string());
        let email_confirmation_template_name = figment
            .extract_inner("email_confirmation_template_name")
            .unwrap_or("email_confirmation".to_string());

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
            .unwrap_or(host.clone());

        let notification_email = figment.extract_inner("notification_email").ok();

        let new_member_notification_template = figment
            .extract_inner("new_member_notification_template")
            .unwrap_or("member_registered".to_string());

        Self {
            email_sender,
            mandrill_api_host,
            email_from_email,
            email_from_name,
            host,
            email_confirmation_template_name,
            postgres,
            web_db_pool,
            processing_db_pool,
            tex_exe,
            processing_queue_size,
            verify_redirects_to,
            notification_email,
            new_member_notification_template,
        }
    }
}
