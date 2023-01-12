use chrono::{Duration, Timelike, Utc};
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

#[derive(Debug, Clone)]
pub struct WakaTime {
    client: Client,
    base_url: Url,
    api_key: String,
}

impl WakaTime {
    pub fn new(api_key: &str) -> Result<Self, Box<dyn Error>> {
        let base_url = "https://wakatime.com".parse()?;
        let api_key = api_key.to_string();

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
    pub async fn get_time_last_n_days(&self, days: i64) -> Result<Option<f64>, Box<dyn Error>> {
        let start_date = Self::get_start_datetime(days);
        let end_date = Self::get_end_datetime();

        let endpoint = "/api/v1/users/current/summaries";
        let url = self.base_url.join(endpoint)?;

        let response = self
            .client
            .get(url)
            .query(&vec![
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

    /// Gets the start date as an ISO string for the WakaTime API request.
    ///
    /// Uses the `chrono` crate to get the current date and subtract the number of days.
    fn get_start_datetime(days: i64) -> String {
        let mut datetime = Utc::now() - Duration::days(days);

        // remove the nanoseconds from the datetime
        datetime = datetime.with_nanosecond(0).unwrap();

        // return converted to RFC 3339 and ISO 8601 string (e.g. 2022-12-07T00:00:00+00:00)
        datetime.to_rfc3339()
    }

    /// Gets the end date as an ISO string for the WakaTime API request.
    fn get_end_datetime() -> String {
        let mut datetime = Utc::now();

        // remove the nanoseconds from the datetime
        datetime = datetime.with_nanosecond(0).unwrap();

        // return converted to RFC 3339 and ISO 8601 string (e.g. 2022-12-07T00:00:00+00:00)
        datetime.to_rfc3339()
    }
}

#[cfg(test)]
mod tests {
    use super::WakaTime;
    use chrono::{Duration, NaiveDate};
    use httpmock::prelude::*;
    use serde_json::json;
    use std::error::Error;

    #[test]
    fn test_get_start_and_end_dates() -> Result<(), Box<dyn Error>> {
        let start = WakaTime::get_start_datetime(30);
        let end = WakaTime::get_end_datetime();

        let start_date = NaiveDate::parse_from_str(&start[..10], "%Y-%m-%d")?;
        let end_date = NaiveDate::parse_from_str(&end[..10], "%Y-%m-%d")?;

        assert_eq!(start_date, end_date - Duration::days(30));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_time_last_n_days() -> Result<(), Box<dyn Error>> {
        let mock_server = MockServer::start();

        let start_date = WakaTime::get_start_datetime(30);
        let end_date = WakaTime::get_end_datetime();

        let mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/api/v1/users/current/summaries")
                .query_param_exists("api_key")
                .query_param("start", &start_date)
                .query_param("end", &end_date);
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

        let mut client = WakaTime::new("test")?;

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
