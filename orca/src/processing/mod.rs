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

use crate::data::{Id, Member};
use crate::db::{DbPool, QueryAs};
use crate::media::ImageData;

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

pub fn start(capacity: usize, db_pool: DbPool) -> QueueSender {
    let (sender, mut receiver) = mpsc::channel(capacity);

    tokio::spawn(async move {
        while let Some(cmd) = receiver.recv().await {
            let res = process(cmd, &db_pool).await;
            println!("Processing result: {res:?}");
        }
    });

    QueueSender(sender)
}

async fn process(command: Command, db_pool: &DbPool) -> Result<(), io::Error> {
    use Command::*;
    match command {
        NewMemberRegistered(member_id, signature) => {
            println!("member added {member_id:?}");

            let processing_dir = format!("data/members/{member_id}");
            fs::create_dir_all(&processing_dir).await?;
            signature
                .write_to_disk(&processing_dir, "signature")
                .await?;

            print_pdf(member_id, &processing_dir, db_pool).await?;

            Ok(())
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MemberDetails {
    first_name: String,
    last_name: String,
    date_of_birth: Date,
    phone_number: String,
    email: String,
    address: String,
    city: String,
    postal_code: String,
    company_name: String,
    occupation: String,
}

async fn print_tex_header(id: Id<Member>, db_pool: &DbPool) -> Result<String, sqlx::Error> {
    let details = query_member(id).fetch_one(db_pool).await?;
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
        &details.first_name,
        &details.last_name,
        details.date_of_birth,
        &details.phone_number,
        &details.email,
        &details.address,
        &details.city,
        &details.postal_code,
        &details.company_name,
        &details.occupation,
        &signature
    ))
}

async fn print_pdf(
    member_id: Id<Member>,
    dir: &str,
    db_pool: &DbPool,
) -> Result<String, io::Error> {
    // inline static files
    let form_tex = include_str!("../../latex/registration.tex");
    let logo_png = include_bytes!("../../latex/logo.png");

    // Write data file for tex
    let tex_header = print_tex_header(member_id, db_pool)
        .await
        .expect("db query should be fine");
    let mut tex_file = fs::File::create(format!("{dir}/data.tex")).await?;
    tex_file.write_all(tex_header.as_bytes()).await?;
    tex_file.flush().await?;

    // Write static content to directory
    fs::write(format!("{dir}/registration.tex"), form_tex).await?;
    fs::write(format!("{dir}/logo.png"), logo_png).await?;

    // Spawn xelatex process to print the pdf
    let mut child = process::Command::new("xelatex")
        .current_dir(dir)
        .arg("registration.tex")
        .arg("-halt-on-error")
        .arg("-shell-escape")
        .stdout(Stdio::null())
        .spawn()?;

    // Await until the command completes
    let status = child.wait().await?;
    println!("the command exited with: {status}");

    Ok("foo".to_string())
}

fn query_member<'a>(id: Id<Member>) -> QueryAs<'a, MemberDetails> {
    sqlx::query_as("
select first_name, last_name, date_of_birth, phone_number, email, address, city, postal_code, company_name, occupation
from members where id = $1
")
    .bind(id)
}
