use std::io::Write;

use chrono::prelude::*;
use chrono_tz::Tz;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

lazy_static::lazy_static! {
    static ref TIMEZONE: Tz = "Europe/Belgrade".parse().unwrap();
}

pub fn current_date_time() -> (String, String) {
    let now: DateTime<Tz> = Utc::now().with_timezone(&TIMEZONE);
    let locale_date: String = format!("{}", now.format("%d.%m.%Y"));
    let locale_time: String = format!("{}", now.format("%H:%M:%S"));
    return (locale_date, locale_time);
}

pub async fn log<T: Into<String>>(log_type: &str, raw_message: T) {
    let message: String = raw_message.into();
    let now: DateTime<Tz> = Utc::now().with_timezone(&TIMEZONE);

    let locale_date: String = format!("{}", now.format("%d.%m.%Y"));
    let locale_time: String = format!("{}", now.format("%H:%M:%S"));

    let message: String = format!("[{}] {} {}\n", log_type, locale_time, message);
    print!("{}", message);

    let file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(format!("./logs/{}.txt", locale_date))
                    .await;
    if let Ok(mut writeable_file) = file {
        if let Err(err) = writeable_file.write_all(message.as_bytes()).await {
            eprintln!("[FILE] {}", err);
        }
    }
}

pub fn log_sync<T: Into<String>>(log_type: &str, raw_message: T) {
    let message: String = raw_message.into();
    let now: DateTime<Tz> = Utc::now().with_timezone(&TIMEZONE);

    let locale_date: String = format!("{}", now.format("%d.%m.%Y"));
    let locale_time: String = format!("{}", now.format("%H:%M:%S"));

    let message: String = format!("[{}] {} {}\n", log_type, locale_time, message);
    print!("{}", message);

    let file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(format!("./logs/{}.txt", locale_date));
    if let Ok(mut writeable_file) = file {
        if let Err(err) = writeable_file.write_all(message.as_bytes()) {
            eprintln!("[FILE] {}", err);
        }
    }
}