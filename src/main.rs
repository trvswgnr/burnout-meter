fn main() {}

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
