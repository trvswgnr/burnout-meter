extern crate chrono;
extern crate reqwest;
extern crate reqwest_oauth1;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate dotenv_codegen;

use chrono::{Duration, Utc};
use reqwest::{Client, Method, Url};
use reqwest_oauth1::OAuthClientProvider;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Summary {
    cummulative_total: Option<Total>,
}

#[derive(Deserialize, Debug)]
struct Total {
    decimal: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct Update {
    location: Option<String>,
}

struct Data {
    update: Update,
    hours: Option<f64>,
    meter: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let data = run().await?;
    if data.update.location.is_some() {
        println!("Updated location to: {}", data.update.location.unwrap());
    } else {
        println!("Failed to update location");
    }
    let hours = data.hours.unwrap_or(0f64);
    println!("{} hours in the last 30 days", hours);
    println!("{} hours to burnout", 170f64 - hours);
    println!("Meter: {}", data.meter);
    Ok(())
}

async fn run() -> Result<Data, Box<dyn Error>> {
    let hours = get_time_last_n_days(30, None).await?;
    let meter = create_meter(hours, 170f64, 8)?;
    let location = format!("{} to burnout", meter);
    let updated = update_twitter_profile_location(&location, None).await?;

    Ok(Data {
        update: updated,
        hours,
        meter,
    })
}

async fn get_time_last_n_days(
    days: i64,
    base_url: Option<Url>,
) -> Result<Option<f64>, Box<dyn Error>> {
    let start_date = (Utc::now() - Duration::days(days)).to_rfc3339();
    let end_date = Utc::now().to_rfc3339();

    let api_key = dotenv!("WAKATIME_API_KEY");
    let base_url = match base_url {
        Some(url) => Some(url),
        None => "https://wakatime.com".parse().ok(),
    };

    if cfg!(test) && base_url == "https://wakatime.com".parse().ok() {
        panic!("base_url is set to wakatime.com in tests");
    }

    let endpoint = "/api/v1/users/current/summaries";
    let url = base_url.map(|url| url.join(endpoint).unwrap()).unwrap();

    let client = Client::new();

    let response = client
        .get(url)
        .query(&[
            ("api_key", &api_key),
            ("start", &start_date.as_str()),
            ("end", &end_date.as_str()),
        ])
        .send()
        .await?;

    let body = response.text().await?;
    let result: Summary = from_str(&body)?;

    match result.cummulative_total {
        Some(total) => Ok(Some(total.decimal.parse()?)),
        None => Ok(None),
    }
}

struct Secrets {
    consumer_key: &'static str,
    consumer_secret: &'static str,
    token: &'static str,
    token_secret: &'static str,
}

impl Secrets {
    fn new() -> Secrets {
        let consumer_key = dotenv!("TWITTER_APP_API_KEY");
        let consumer_secret = dotenv!("TWITTER_APP_API_SECRET_KEY");
        let token = dotenv!("TWITTER_APP_ACCESS_TOKEN");
        let token_secret = dotenv!("TWITTER_APP_ACCESS_TOKEN_SECRET");
        Secrets {
            consumer_key,
            consumer_secret,
            token,
            token_secret,
        }
    }
}

async fn update_twitter_profile_location(
    location: &str,
    base_url: Option<Url>,
) -> Result<Update, Box<dyn Error>> {
    let Secrets {
        consumer_key,
        consumer_secret,
        token,
        token_secret,
    } = Secrets::new();

    let base_url = match base_url {
        Some(url) => Some(url),
        None => "https://api.twitter.com".parse().ok(),
    };

    if cfg!(test) && base_url == "https://api.twitter.com".parse().ok() {
        panic!("base_url is set to api.twitter.com in tests");
    }

    let endpoint = "/1.1/account/update_profile.json";

    let url = base_url.map(|url| url.join(endpoint).unwrap()).unwrap();

    let client = Client::new();

    let secrets =
        reqwest_oauth1::Secrets::new(consumer_key, consumer_secret).token(token, token_secret);

    let response = client
        .oauth1(secrets)
        .request(Method::POST, url)
        .form(&[("location", location)]);

    let response = response.send().await?;

    let body = response.text().await?;
    let result: Result<Update, _> = from_str(&body);

    assert!(result.is_ok());

    Ok(result.unwrap())
}

fn create_meter(current: Option<f64>, max: f64, length: u8) -> Result<String, Box<dyn Error>> {
    if current.is_none() {
        let meter = "⬜️".repeat(length as usize);
        return Ok(meter);
    }

    let current = current.unwrap();

    // get the current without the decimal
    let hours = current.floor();
    let remainder = current - hours;
    let percentage = current / max;
    let mut filled = (percentage * length as f64).floor() as u8;

    if remainder > 0.5 && length >= 10 && filled < length {
        filled += 1;
    }

    let empty = length - filled;

    let mut emoji = String::from("🟩");

    if percentage > 0.45 {
        emoji = String::from("🟨");
    }

    if percentage > 0.7 {
        emoji = String::from("🟧");
    }

    if percentage > 0.94 {
        emoji = String::from("🟥");
    }

    let blank = "⬜️";

    let meter = emoji.repeat(filled as usize) + &blank.repeat(empty as usize);

    Ok(meter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;
    use std::error::Error;

    #[tokio::test]
    async fn test_get_time_last_n_days() -> Result<(), Box<dyn Error>> {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/api/v1/users/current/summaries")
                .query_param_exists("api_key")
                .query_param_exists("start")
                .query_param_exists("end");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    json!({
                        "cummulative_total": {
                          "decimal": "155.00",
                          "digital": "155:00",
                          "seconds": 558020.669293,
                          "text": "155 hrs"
                        }
                    })
                    .to_string(),
                );
        });
        let base_url = mock_server.base_url().parse::<Url>().ok();
        let result = get_time_last_n_days(1, base_url).await;

        mock.assert();

        assert!(result.is_ok());

        assert_eq!(result.unwrap().unwrap(), 155f64);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_twitter_profile_location() -> Result<(), Box<dyn Error>> {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(POST)
                .path("/1.1/account/update_profile.json")
                .header("content-type", "application/x-www-form-urlencoded")
                .header_exists("authorization")
                .x_www_form_urlencoded_key_exists("location");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    json!({
                        "location": "San Francisco, CA"
                    })
                    .to_string(),
                );
        });
        let base_url = mock_server.base_url().parse::<Url>().ok();
        let result = update_twitter_profile_location("San Francisco, CA", base_url).await;

        mock.assert();

        assert!(result.is_ok());

        assert_eq!(
            result.unwrap().location,
            Some("San Francisco, CA".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_create_meter() -> Result<(), Box<dyn Error>> {
        let meter = create_meter(Some(0f64), 170f64, 8)?;
        assert_eq!(meter, "⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️");

        let meter = create_meter(Some(1f64), 10f64, 10)?;
        assert_eq!(meter, "🟩⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️");

        let meter = create_meter(Some(5.5f64), 10f64, 10)?;
        assert_eq!(meter, "🟨🟨🟨🟨🟨⬜️⬜️⬜️⬜️⬜️");

        let meter = create_meter(Some(7.1f64), 10f64, 10)?;
        assert_eq!(meter, "🟧🟧🟧🟧🟧🟧🟧⬜️⬜️⬜️");

        let meter = create_meter(Some(9.4f64), 10f64, 10)?;
        assert_eq!(meter, "🟥🟥🟥🟥🟥🟥🟥🟥🟥⬜️");

        let meter = create_meter(Some(9.4f64), 10f64, 5)?;
        assert_eq!(meter, "🟥🟥🟥🟥⬜️");

        let meter = create_meter(Some(9.4f64), 10f64, 20)?;
        assert_eq!(meter, "🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥⬜️⬜️");

        Ok(())
    }
}
