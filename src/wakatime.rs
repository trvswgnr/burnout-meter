use chrono::{Duration, Utc};
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::from_str;
use std::error::Error;

use crate::util::get_env_var;

#[derive(Deserialize, Debug)]
struct Summary {
    cummulative_total: Option<Total>,
}

#[derive(Deserialize, Debug)]
struct Total {
    decimal: String,
}

#[derive(Debug, Clone)]
pub(crate) struct WakaTimeClient {
    client: Client,
    base_url: Url,
    api_key: String,
}

impl WakaTimeClient {
    pub(crate) fn new() -> Result<Self, Box<dyn Error>> {
        let api_key = get_env_var("WAKATIME_API_KEY")?;
        let base_url = "https://wakatime.com".parse()?;

        Ok(Self {
            client: Client::new(),
            base_url,
            api_key,
        })
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
        &self,
        days: i64,
    ) -> Result<Option<f64>, Box<dyn Error>> {
        let start_date = (Utc::now() - Duration::days(days)).to_rfc3339();
        let end_date = Utc::now().to_rfc3339();

        let endpoint = "/api/v1/users/current/summaries";
        let url = self.base_url.join(endpoint)?;

        let response = self
            .client
            .get(url)
            .query(&[
                ("api_key", &self.api_key),
                ("start", &start_date),
                ("end", &end_date),
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
}

#[cfg(test)]
mod tests {
    use super::WakaTimeClient;
    use httpmock::prelude::*;
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

        let mut client = WakaTimeClient::new()?;

        client.base_url = mock_server.base_url().parse()?;
        let response = client.get_time_last_n_days(30).await;

        assert!(
            response.is_ok(),
            "get_time_last_n_days failed: {}",
            response.unwrap_err()
        );

        let result = response.unwrap();

        // ensure the mock server was called exactly one time
        mock.assert();

        assert_eq!(result, Some(155f64));

        Ok(())
    }
}
