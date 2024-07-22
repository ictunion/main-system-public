//! This is done using `tokio::sync::mpsc` channels
//! We spam to run outside of request processing threads (workers).
//! This also might be called `queue` or `worker`
use phf::phf_map;

use time::Date;
use tokio::fs;
use tokio::io::{self, AsyncWriteExt};
use tokio::process;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use lettre::{
    address, message::Attachment, message::Mailbox, message::MultiPart, message::SinglePart,
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use std::process::Stdio;

use crate::config::templates;
use crate::config::Config;
use crate::data::{Id, RegistrationRequest};
use crate::db::DbPool;
use crate::media::{ImageData, TexEscape};

mod query;

#[derive(Debug)]
pub enum Command {
    NewRegistrationRequest(Id<RegistrationRequest>, ImageData),
    RegistrationRequestVerified(Id<RegistrationRequest>),
    ResentRegistrationEmail(Id<RegistrationRequest>),
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::NewRegistrationRequest(id, _) => {
                write!(f, "NewRegistrationRequest id: {id}")
            }
            Self::RegistrationRequestVerified(id) => {
                write!(f, "RegistrationRequestVerified id: {id}")
            }
            Self::ResentRegistrationEmail(id) => {
                write!(f, "ResentRegistrationEmail id: {id}")
            }
        }
    }
}

pub struct QueueSender(Sender<Command>);

#[derive(Debug)]
pub enum SenderError {
    Mpsc(tokio::sync::mpsc::error::SendError<Command>),
}

impl QueueSender {
    pub async fn send(&self, cmd: Command) -> Result<(), SenderError> {
        self.0.send(cmd).await.map_err(SenderError::Mpsc)
    }
}

pub fn start(config: &Config, db_pool: DbPool) -> QueueSender {
    let (sender, mut receiver) = mpsc::channel::<Command>(config.processing_queue_size);
    let our_conf = config.clone();

    info!("Starting processing queue");
    tokio::spawn(async move {
        while let Some(cmd) = receiver.recv().await {
            let cmd_info = cmd.to_string();
            info!("Processiong command: {cmd_info}");
            match process(cmd, &our_conf, &db_pool).await {
                Ok(()) => info!("Command processed successfully"),
                Err(err) => error!("Processing of cmd: {cmd_info} failed with: {:?}", err),
            }
        }
    });

    QueueSender(sender)
}

#[derive(Debug)]
enum ProcessingError {
    Io(io::Error),
    Sql(sqlx::Error),
    EmailAddress(address::AddressError),
    EmailRendering(handlebars::RenderError),
    Lettre(lettre::error::Error),
    Smtp(lettre::transport::smtp::Error),
    MissingConfirmationToken,
}

impl From<io::Error> for ProcessingError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<sqlx::Error> for ProcessingError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sql(value)
    }
}

impl From<address::AddressError> for ProcessingError {
    fn from(value: address::AddressError) -> Self {
        Self::EmailAddress(value)
    }
}

impl From<lettre::error::Error> for ProcessingError {
    fn from(value: lettre::error::Error) -> Self {
        Self::Lettre(value)
    }
}

impl From<handlebars::RenderError> for ProcessingError {
    fn from(value: handlebars::RenderError) -> Self {
        Self::EmailRendering(value)
    }
}

impl From<lettre::transport::smtp::Error> for ProcessingError {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        Self::Smtp(value)
    }
}

async fn process(
    command: Command,
    config: &Config,
    db_pool: &DbPool,
) -> Result<(), ProcessingError> {
    use Command::*;

    match command {
        NewRegistrationRequest(reg_id, signature) => {
            process_new_registration(reg_id, signature, config, db_pool).await?;
        }
        ResentRegistrationEmail(reg_id) => {
            let application_details = query::query_registration(reg_id).fetch_one(db_pool).await?;

            let (pdf_data,) = query::fetch_registration_pdf(reg_id)
                .fetch_one(db_pool)
                .await?;

            send_verification_email(config, db_pool, &application_details, pdf_data).await?;
        }
        RegistrationRequestVerified(reg_id) => {
            // We do this only if notification email is configured
            if let Some(notification_email) = &config.notification_email {
                let sender_info: Mailbox = format!(
                    "{} <{}>",
                    config.email_sender_name.clone().unwrap_or("".to_string()),
                    config.email_sender_email
                )
                .parse()?;

                let mut renderer = config
                    .templates
                    .renderer(&templates::NEW_APPLICATION_NOTICE, "default");

                // We need member details because we want to customize email subject/pdf name
                let application_details =
                    query::query_registration(reg_id).fetch_one(db_pool).await?;

                let application_detail_url = format!(
                    "{}/applications/{}",
                    config.admin_host.as_deref().unwrap_or(""),
                    application_details.id
                );

                renderer.bind("application_detail_url", &application_detail_url);

                let message_html = config.templates.render(&renderer)?;

                let email_subject = format!(
                    "New Application - {} {} from {}",
                    application_details.first_name.as_deref().unwrap_or(""),
                    application_details.last_name.as_deref().unwrap_or(""),
                    application_details.company_name.as_deref().unwrap_or("")
                );

                let message = Message::builder()
                    .from(sender_info.clone())
                    .reply_to(sender_info)
                    .to(format!("Notifications <{}>", notification_email).parse()?)
                    .subject(email_subject)
                    .multipart(MultiPart::related().singlepart(SinglePart::html(message_html)))?;

                send_email(config, message).await?;
            }
        }
    }

    Ok(())
}

async fn send_email(config: &Config, message: Message) -> Result<(), ProcessingError> {
    let creds = Credentials::new(config.smtp_user.to_owned(), config.smtp_password.to_owned());
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
            .credentials(creds)
            .build();

    mailer.send(message).await?;
    Ok(())
}

async fn process_new_registration(
    reg_id: Id<RegistrationRequest>,
    signature: ImageData,
    config: &Config,
    db_pool: &DbPool,
) -> Result<(), ProcessingError> {
    // Prepare directory & image data for processing
    let processing_dir = format!("data/registration_requests/{reg_id}");
    fs::create_dir_all(&processing_dir).await?;
    signature
        .write_to_disk(&processing_dir, "signature")
        .await?;

    // Query for detail information about member
    // in theory we could also pass this in the command
    // but doing it this way means that all triggers & defaults etc are 100% applied to the data
    let application_details = query::query_registration(reg_id).fetch_one(db_pool).await?;

    // Generate PDF
    let pdf_path = print_pdf(config, &application_details, &processing_dir).await?;
    let pdf_data = fs::read(pdf_path).await?;
    query::insert_registration_pdf(reg_id, &pdf_data)
        .execute(db_pool)
        .await?;

    send_verification_email(config, db_pool, &application_details, pdf_data).await?;

    // Remove directory containing data for processing
    fs::remove_dir_all(processing_dir).await?;

    Ok(())
}

async fn send_verification_email(
    config: &Config,
    db_pool: &DbPool,
    application_details: &RegistrationDetails,
    pdf_data: Vec<u8>,
) -> Result<(), ProcessingError> {
    info!("Send verification email to {}", application_details.email);

    let token = application_details
        .confirmation_token
        .as_ref()
        .ok_or(ProcessingError::MissingConfirmationToken)?;

    let verify_link = format!("{}/registration/{}/confirm", config.host, token,);

    let sender_info: Mailbox = format!(
        "{} <{}>",
        config.email_sender_name.clone().unwrap_or("".to_string()),
        config.email_sender_email
    )
    .parse()?;

    let full_name = format!(
        "{} {}",
        application_details.first_name.as_deref().unwrap_or(""),
        application_details.last_name.as_deref().unwrap_or("")
    );
    let subject =
        config.email_confirmation_subject_for_local(&application_details.registration_local);

    let mut renderer = config.templates.renderer(
        &templates::EMAIL_VERIFICATION,
        &application_details.registration_local,
    );

    renderer
        .bind(
            "first_name",
            application_details.first_name.as_deref().unwrap_or(""),
        )
        .bind(
            "last_name",
            application_details.last_name.as_deref().unwrap_or(""),
        )
        .bind("verify_link", &verify_link);

    let message_html = config.templates.render(&renderer)?;

    let message = Message::builder()
        .from(sender_info.clone())
        .reply_to(sender_info)
        .to(format!("{} <{}>", full_name, application_details.email).parse()?)
        .subject(subject)
        .multipart(
            MultiPart::related()
                .singlepart(SinglePart::html(message_html))
                .singlepart(
                    Attachment::new(String::from("application.pdf"))
                        // This should never fail
                        // we generate the pdf ourselves so we know it will be valid
                        .body(pdf_data, "application/pdf".parse().unwrap()),
                ),
        )?;

    send_email(config, message).await?;

    // Update info in DB about email being sent
    query::track_verification_sent_at(application_details.id)
        .execute(db_pool)
        .await?;

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
pub struct RegistrationDetails {
    pub id: Id<RegistrationRequest>,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<Date>,
    pub phone_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub company_name: Option<String>,
    pub occupation: Option<String>,
    pub confirmation_token: Option<String>,
    pub registration_local: String,
}

async fn print_tex_header(details: &RegistrationDetails) -> Result<String, ProcessingError> {
    let signature = format!(
        "{} {}",
        details.first_name.as_deref().unwrap_or(""),
        details.last_name.as_deref().unwrap_or("")
    );
    Ok(format!(
        "\
\\def\\Name{{{}}}
\\def\\Surname{{{}}}
\\def\\DateOfBirth{{{}}}
\\def\\Phone{{{}}}
\\def\\Email{{{}}}
\\def\\Address{{{}}}
\\def\\City{{{}}}
\\def\\Zipcode{{{}}}
\\def\\Company{{{}}}
\\def\\Position{{{}}}
\\def\\Signature{{{}}}
",
        details.first_name.as_deref().escape_tex(),
        details.last_name.as_deref().escape_tex(),
        // This doesn't need to be escaped because
        // Date is not arbitrary string of characters.
        details.date_of_birth.escape_tex(),
        details.phone_number.as_deref().escape_tex(),
        details.email.as_str().escape_tex(),
        details.address.as_deref().escape_tex(),
        details.city.as_deref().escape_tex(),
        details.postal_code.as_deref().escape_tex(),
        details.company_name.as_deref().escape_tex(),
        details.occupation.as_deref().escape_tex(),
        &signature.escape_tex()
    ))
}

const DEFAULT_PDF_TRANSLATION: &str = include_str!("../../latex/lang.en.tex");
static PDF_LOCALIZATIONS: phf::Map<&'static str, &'static str> = phf_map!(
    "en" => DEFAULT_PDF_TRANSLATION,
    "cs" => include_str!("../../latex/lang.cs.tex"),
);

fn get_pdf_localization(lang: &str) -> &'static str {
    PDF_LOCALIZATIONS
        .get(lang)
        .unwrap_or(&DEFAULT_PDF_TRANSLATION)
}

/// Given a member id, directory and db_pool
/// This will generate registration PDF for
/// application using xelatex and return path to the PDF file
async fn print_pdf(
    config: &Config,
    application_details: &RegistrationDetails,
    dir: &str,
) -> Result<String, ProcessingError> {
    info!("Printing registration pdf at {dir}");

    // inline static files
    let form_tex = include_str!("../../latex/registration.tex");
    let logo_png = include_bytes!("../../latex/logo.png");

    // Write data file for tex
    let tex_header = print_tex_header(application_details).await?;
    let mut tex_file = fs::File::create(format!("{dir}/data.tex")).await?;
    tex_file.write_all(tex_header.as_bytes()).await?;
    tex_file.flush().await?;

    // Write static content to directory
    fs::write(format!("{dir}/registration.tex"), form_tex).await?;
    fs::write(format!("{dir}/registration.tex"), form_tex).await?;
    fs::write(
        format!("{dir}/lang.tex"),
        get_pdf_localization(&application_details.registration_local),
    )
    .await?;
    fs::write(format!("{dir}/logo.png"), logo_png).await?;

    // Spawn xelatex process to print the pdf
    let mut child = process::Command::new(config.tex_exe.as_str())
        .current_dir(dir)
        .arg("registration.tex")
        .arg("-halt-on-error")
        .arg("-no-shell-escape")
        .stdout(Stdio::null())
        .spawn()?;

    // Await until command completes
    // There is a problem with tokio detecting the exist status of the process
    // at least in cases where xelatex fails to find font in OSFONTDIR
    // like similar to https://users.rust-lang.org/t/tokio-child-wait-never-returning/96657
    let status = child.wait().await?;
    info!("Tex command exited successfully: {status}");

    Ok(format!("{dir}/registration.pdf"))
}
