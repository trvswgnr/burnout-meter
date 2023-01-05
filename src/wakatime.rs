use chrono::{Duration, Utc};
use reqwest::{Client, Url};
use serde::Deserialize;
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

/// Get the total time spent coding in the last n days
/// from the WakaTime API
///
/// https://wakatime.com/developers#summaries
///
/// # Errors
///
/// Returns an error if the request fails or the response cannot be parsed.
/// Returns an error if the `base_url` is set to `https://wakatime.com` when running tests.
///
/// # Examples
///
/// ```no_run
/// use wakatime::get_time_last_n_days;
/// use std::error::Error;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn Error>> {
///     let hours = get_time_last_n_days(30, None).await?;
///     println!("{} hours in the last 30 days", hours.unwrap_or(0f64));
///     Ok(())
/// }
/// ```
pub(crate) async fn get_time_last_n_days(
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

    if cfg!(test) && base_url == "https://wakatime.com/api/v1".parse().ok() {
        return Err("base_url is set to wakatime.com in tests".into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;

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
}
