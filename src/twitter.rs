use reqwest::{Client, Method, Url};
use reqwest_oauth1::OAuthClientProvider;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::error::Error;

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct Update {
    pub location: Option<String>,
}

struct Secrets {
    pub(crate) consumer_key: &'static str,
    pub(crate) consumer_secret: &'static str,
    pub(crate) token: &'static str,
    pub(crate) token_secret: &'static str,
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

/// Update the location field in your Twitter profile from the Twitter API
///
/// https://developer.twitter.com/en/docs/accounts-and-users/manage-account-settings/api-reference/post-account-update_profile
///
/// # Errors
///
/// Returns an error if the request fails or the response cannot be parsed.
/// Returns an error if the `base_url` is set to `https://api.twitter.com` when running tests.
///
/// # Examples
///
/// ```no_run
/// use twitter::update_twitter_profile_location;
/// use std::error::Error;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn Error>> {
///     let location = "San Francisco, CA";
///     let result = update_twitter_profile_location(location, None).await?;
///     println!("Updated location to {}", result.location.unwrap());
///     Ok(())
/// }
/// ```
pub(crate) async fn update_twitter_profile_location(
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
        return Err("base_url is set to https://api.twitter.com in tests".into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;
    use std::error::Error;

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
}
