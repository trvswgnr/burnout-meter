use dotenv::dotenv;
use std::{env, error::Error, sync::Once};

static INIT: Once = Once::new();

pub(crate) fn get_env_var(key: &str) -> Result<String, Box<dyn Error>> {
    INIT.call_once(|| match dotenv().ok() {
        Some(_) => println!(".env file detected, loading..."),
        None => println!("No .env file found."),
    });

    match env::var(key) {
        Ok(val) => Ok(val),
        Err(_) => Err(format!("{key} not set").into()),
    }
}
