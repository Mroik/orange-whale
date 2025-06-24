use std::{
    env::var,
    fs::{File, remove_file},
    io::Read,
    time::{Duration, UNIX_EPOCH},
};

use anyhow::Result;
use clokwerk::{AsyncScheduler, Interval};
use log::{debug, error, info};
use rpgpie::{
    certificate::Certificate,
    message::{SignatureMode, encrypt},
    policy::Seipd,
};
use tar::Builder;
use teloxide::{Bot, prelude::Requester, types::InputFile};
use tokio::time::sleep;

const MAX_FILE_SIZE: usize = 50000000;
const BACKUP: &str = "backup";
const BACKUP_ENCRYPTED: &str = "backup_encrypted";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let mut scheduler = AsyncScheduler::new();
    let interval = var("INTERVAL")?.parse::<u32>()?;

    scheduler.every(Interval::Hours(interval)).run(|| async {
        backup().await.unwrap();
    });

    info!("Program set to backup every {} hours", interval);

    info!("Running first backup out of schedule");
    backup().await?;

    loop {
        scheduler.run_pending().await;
        sleep(Duration::from_secs(30)).await;
    }
}

async fn backup() -> Result<()> {
    info!("Running a backup");
    let chat_id = var("CHAT_ID").unwrap();
    let locations: Vec<String> = var("LOCATIONS")
        .unwrap()
        .trim()
        .split(':')
        .map(|s| s.to_string())
        .collect();

    archive(&locations)?;
    encrypt_data("pub.txt")?;
    remove_file(BACKUP)?;

    let bot = Bot::from_env();
    let mut data_to_send = File::open(BACKUP_ENCRYPTED)?;

    bot.send_message(
        chat_id.clone(),
        format!("{}", UNIX_EPOCH.elapsed()?.as_secs().to_string()),
    )
    .await?;

    let mut rr;
    let mut part = 0;
    loop {
        let mut buf = vec![0; MAX_FILE_SIZE];
        rr = data_to_send.read(&mut buf)?;
        debug!("Read {} bytes", rr);

        let data = InputFile::memory(buf).file_name(format!("part_{:06}", part));

        if rr == 0 {
            break;
        }

        loop {
            match bot.send_document(chat_id.clone(), data.clone()).await {
                Ok(_) => break,
                Err(teloxide::RequestError::RetryAfter(n)) => {
                    error!("Awaiting {} seconds", n.seconds());
                    sleep(Duration::from_secs(n.seconds() as u64)).await
                }
                Err(err) => {
                    error!("{}\nAwaiting 8 seconds", err);
                    sleep(Duration::from_secs(8)).await
                }
            }
        }
        info!("Sent part {}", part);
        part += 1;
    }

    drop(data_to_send);
    remove_file(BACKUP_ENCRYPTED)?;

    info!("Backup terminated");
    Ok(())
}

fn archive(location: &[String]) -> Result<()> {
    info!("Creating archive");
    let mut b = Builder::new(File::create(BACKUP)?);
    for loc in location {
        b.append_dir_all(loc, loc)?;
    }
    b.finish()?;
    info!("Archive created");
    Ok(())
}

fn encrypt_data(key_file: &str) -> Result<()> {
    info!("Encrypting archive");
    let rec = Certificate::load(&mut File::open(key_file)?)?;
    let mut plaintext = File::open(BACKUP)?;
    let mut output = File::create(BACKUP_ENCRYPTED)?;
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
    info!("Archive encrypted");
    Ok(())
}
