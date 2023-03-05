//! Implement async processing of data
//! This is done using `tokio::sync::mpsc` channels
//! We spam to run outside of request processing threads (workers).
//! This also might be called `queue` or `worker`

use time::Date;
use tokio::fs;
use tokio::io::{self, AsyncWriteExt};
use tokio::process;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use std::process::Stdio;

mod mandrill;
use mandrill::{Attachement, TemplateMessage};

use crate::config::Config;
use crate::data::{Id, Member};
use crate::db::{DbPool, Query, QueryAs};
use crate::media::{ImageData, TexEscape};

use self::mandrill::TemplateContentItem;

#[derive(Debug)]
pub enum Command {
    NewMemberRegistered(Id<Member>, ImageData),
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
    let email_sender: mandrill::Sender = mandrill::Sender::new(config);
    let (sender, mut receiver) = mpsc::channel(config.processing_queue_size);
    let our_conf = config.clone();

    tokio::spawn(async move {
        while let Some(cmd) = receiver.recv().await {
            let res = process(cmd, &our_conf, &db_pool, &email_sender).await;
            println!("Processing result: {res:?}");
        }
    });

    QueueSender(sender)
}

#[derive(Debug)]
enum ProcessingError {
    Io(io::Error),
    Sql(sqlx::Error),
    EmailSender(mandrill::SenderError),
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

impl From<mandrill::SenderError> for ProcessingError {
    fn from(value: mandrill::SenderError) -> Self {
        Self::EmailSender(value)
    }
}

async fn process(
    command: Command,
    config: &Config,
    db_pool: &DbPool,
    email_sender: &mandrill::Sender,
) -> Result<(), ProcessingError> {
    use Command::*;
    match command {
        NewMemberRegistered(member_id, signature) => {
            process_new_member_registered(member_id, signature, config, db_pool, email_sender).await
        }
    }
}

async fn process_new_member_registered(
    member_id: Id<Member>,
    signature: ImageData,
    config: &Config,
    db_pool: &DbPool,
    email_sender: &mandrill::Sender,
) -> Result<(), ProcessingError> {
    // Prepare directory & image data for processing
    let processing_dir = format!("data/members/{member_id}");
    fs::create_dir_all(&processing_dir).await?;
    signature
        .write_to_disk(&processing_dir, "signature")
        .await?;

    // Query for detail information about member
    // in theory we could also pass this in thje command
    // but doing it this way means that all triggers, defaults etc are 100% applied to the data
    let member_details = query_member(member_id).fetch_one(db_pool).await?;

    // Generate PDF
    let pdf_path = print_pdf(config, &member_details, &processing_dir).await?;
    let pdf_data = fs::read(pdf_path).await?;
    insert_registration_pdf(member_id, &pdf_data)
        .execute(db_pool)
        .await?;

    // Send email
    let verify_url = format!(
        "{}/registration/{}/confirm",
        config.host, member_details.confirmation_token
    );
    let pdf_attachement = Attachement::new("registration.pdf", "application/pdf", &pdf_data);
    let mut message = TemplateMessage::new("Verify Email Address", &member_details, config);

    message
        .attach(pdf_attachement)
        .bind(TemplateContentItem::new(
            "first_name",
            &member_details.first_name,
        ))
        .bind(TemplateContentItem::new(
            "last_name",
            &member_details.last_name,
        ))
        .bind(TemplateContentItem::new("verify_link", &verify_url));

    email_sender
        .send_template(&config.email_confirmation_template_name, message)
        .await?;

    // Update info in DB about email being sent
    track_verification_sent_at(member_id)
        .execute(db_pool)
        .await?;

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
pub struct MemberDetails {
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: Date,
    pub phone_number: String,
    pub email: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub company_name: String,
    pub occupation: String,
    pub confirmation_token: String,
}

async fn print_tex_header(details: &MemberDetails) -> Result<String, ProcessingError> {
    let signature = format!("{} {}", details.first_name, details.last_name);
    // TODO: implement tex escaping!
    Ok(format!(
        "\
\\newcommand{{\\Name}}{{{}}}
\\newcommand{{\\Surname}}{{{}}}
\\newcommand{{\\DateOfBirth}}{{{}}}
\\newcommand{{\\Phone}}{{{}}}
\\newcommand{{\\Email}}{{{}}}
\\newcommand{{\\Address}}{{{}}}
\\newcommand{{\\City}}{{{}}}
\\newcommand{{\\Zipcode}}{{{}}}
\\newcommand{{\\Company}}{{{}}}
\\newcommand{{\\Position}}{{{}}}
\\newcommand{{\\Signature}}{{{}}}
",
        &details.first_name.escape_tex(),
        &details.last_name.escape_tex(),
        // This doesn't need to be escaped because
        // Date isn't arbitrary string of characters.
        details.date_of_birth,
        &details.phone_number.escape_tex(),
        &details.email.escape_tex(),
        &details.address.escape_tex(),
        &details.city.escape_tex(),
        &details.postal_code.escape_tex(),
        &details.company_name.escape_tex(),
        &details.occupation.escape_tex(),
        &signature.escape_tex()
    ))
}

/// Given a member id, directory and db_pool
/// This will generate registration PDF for
/// application using xelatex and return path to the PDF file
async fn print_pdf(
    config: &Config,
    member_details: &MemberDetails,
    dir: &str,
) -> Result<String, ProcessingError> {
    // inline static files
    let form_tex = include_str!("../../latex/registration.tex");
    let logo_png = include_bytes!("../../latex/logo.png");

    // Write data file for tex
    let tex_header = print_tex_header(member_details).await?;
    let mut tex_file = fs::File::create(format!("{dir}/data.tex")).await?;
    tex_file.write_all(tex_header.as_bytes()).await?;
    tex_file.flush().await?;

    // Write static content to directory
    fs::write(format!("{dir}/registration.tex"), form_tex).await?;
    fs::write(format!("{dir}/logo.png"), logo_png).await?;

    // Spawn xelatex process to print the pdf
    let mut child = process::Command::new(config.tex_exe.as_str())
        .current_dir(dir)
        .arg("registration.tex")
        .arg("-halt-on-error")
        .arg("-no-shell-escape")
        .stdout(Stdio::null())
        .spawn()?;

    // Await until the command completes
    let status = child.wait().await?;
    println!("the command exited with: {status}");

    Ok(format!("{dir}/registration.pdf"))
}

fn query_member<'a>(id: Id<Member>) -> QueryAs<'a, MemberDetails> {
    sqlx::query_as("
select first_name, last_name, date_of_birth, phone_number, email, address, city, postal_code, company_name, occupation, confirmation_token
from members where id = $1
")
    .bind(id)
}

fn insert_registration_pdf(id: Id<Member>, data: &Vec<u8>) -> Query<'_> {
    sqlx::query(
        "
insert into files
( name
, file_type
, data
, user_id
) values ('registration', 'pdf', $1, $2)
",
    )
    .bind(data)
    .bind(id)
}

fn track_verification_sent_at(id: Id<Member>) -> Query<'static> {
    sqlx::query(
        "
update members as m
set verification_sent_at = now()
where id = $1
",
    )
    .bind(id)
}
