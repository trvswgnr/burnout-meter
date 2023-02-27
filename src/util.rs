use dotenv::dotenv;
use std::{env, error::Error, str::FromStr, sync::Once};

static INIT: Once = Once::new();

pub fn get_env_var<T: FromStr>(key: &str) -> Result<T, Box<dyn Error>> {
    INIT.call_once(|| match dotenv().ok() {
        Some(_) => println!(".env file detected, loading..."),
        None => println!("No .env file found."),
    });

    match env::var(key) {
        Ok(val) => {
            let parsed: Result<T, Box<dyn Error>> = match val.parse::<T>() {
                Ok(parsed) => Ok(parsed),
                Err(_) => Err(format!("Failed to parse {key}").into()),
            };

            parsed
        }
        Err(_) => Err(format!("{key} not set").into()),
    }
}

pub fn days_since_monday(offset_hours: i8) -> i64 {
    let offset = time::UtcOffset::from_hms(offset_hours, 0, 0).unwrap_or(time::UtcOffset::UTC);
    let now = time::OffsetDateTime::now_utc().to_offset(offset);
    let monday = now - time::Duration::days(now.weekday().number_from_monday() as i64);
    let days = now - monday;
    days.whole_days()
}
