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
