use std::{
    fs::File,
    io::Cursor,
    time::{Duration, UNIX_EPOCH},
};

use anyhow::Result;
use rpgpie::{
    certificate::Certificate,
    message::{SignatureMode, encrypt},
    policy::Seipd,
};
use tar::Builder;
use teloxide::{Bot, prelude::Requester, types::InputFile};
use tokio::time::sleep;

const MAX_FILE_SIZE: usize = 50000000;

#[tokio::main]
async fn main() -> Result<()> {
    let chat_id = std::env::var("CHAT_ID").unwrap();
    let tar_data = archive("test")?;
    let encrypted_tar = encrypt_data("pub.txt", tar_data)?;

    let bot = Bot::from_env();
    let documents: Vec<InputFile> = encrypted_tar
        .chunks(MAX_FILE_SIZE)
        .enumerate()
        .map(|(part, data)| InputFile::memory(data.to_vec()).file_name(format!("part_{:06}", part)))
        .collect();

    bot.send_message(
        chat_id.clone(),
        format!("{}", UNIX_EPOCH.elapsed()?.as_secs().to_string()),
    )
    .await?;

    for x in documents {
        loop {
            match bot.send_document(chat_id.clone(), x.clone()).await {
                Ok(_) => break,
                Err(teloxide::RequestError::RetryAfter(n)) => {
                    println!("Awaiting {} seconds", n.seconds());
                    sleep(Duration::from_secs(n.seconds() as u64)).await
                }
                Err(err) => {
                    println!("{}\nAwaiting 8 seconds", err);
                    sleep(Duration::from_secs(8)).await
                }
            }
        }
    }

    Ok(())
}

fn archive(location: &str) -> Result<Vec<u8>> {
    let mut b = Builder::new(Vec::new());
    b.append_dir_all(location, location)?;
    Ok(b.into_inner()?)
}

fn encrypt_data(key_file: &str, message: Vec<u8>) -> Result<Vec<u8>> {
    let rec = Certificate::load(&mut File::open(key_file)?)?;
    let mut plaintext = Cursor::new(message);
    let mut output = Cursor::new(Vec::new());
    encrypt(
        Some(Seipd::SEIPD2),
        rec,
        Vec::new(),
        Vec::new(),
        &Vec::new(),
        &mut plaintext,
        SignatureMode::Binary,
        &mut output,
        false,
    )?;
    Ok(output.into_inner())
}
