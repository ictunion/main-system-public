//! Implement async processing of data
//! This is done using `tokio::sync::mpsc` channels
//! We spam to run outside of request processing threads (workers).
//! This also might be called `queue` or `worker`

use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::data::{Id, Member};
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

pub fn start(capacity: usize) -> QueueSender {
    let (sender, mut receiver) = mpsc::channel(capacity);

    tokio::spawn(async move {
        while let Some(cmd) = receiver.recv().await {
            let res = process(cmd).await;
            println!("Processing result: {:?}", res);
        }
    });

    QueueSender(sender)
}

async fn process(command: Command) -> Result<(), tokio::io::Error> {
    use Command::*;
    match command {
        NewMemberRegistered(member_id, signature) => {
            println!("member added {:?}", member_id);

            let processing_dir = format!("data/members/{}", member_id);
            fs::create_dir_all(&processing_dir).await?;
            signature
                .write_to_disk(&processing_dir, "signature")
                .await?;

            let tex_header = print_tex_header();

            let mut tex_file = fs::File::create(format!("{}/registration.tex", processing_dir)).await?;
            tex_file.write_all(&tex_header.as_bytes()).await?;

            // write!(tex_file, "{}", tex_header);
            tex_file.flush().await?;


            Ok(())
        }
    }
}

fn print_tex_header() -> String {
    format!(
        "\
\\newcommand{{\\Name}}{{Jane}}
\\newcommand{{\\Surname}}{{Doe}}
\\newcommand{{\\DateOfBirth}}{{1999-1-2}}
\\newcommand{{\\Phone}}{{+420938281}}
\\newcommand{{\\Email}}{{me@myself.com}}
\\newcommand{{\\Address}}{{Elm Street}}
\\newcommand{{\\City}}{{HK}}
\\newcommand{{\\Zipcode}}{{63100}}
\\newcommand{{\\Company}}{{COZZ}}
\\newcommand{{\\Position}}{{freedom fighter}}
"
    )
}
