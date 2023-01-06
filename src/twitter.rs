use reqwest::{Client, Method, Url};
use reqwest_oauth1::OAuthClientProvider;
use reqwest_oauth1::Secrets;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::error::Error;

use crate::util::get_env_var;

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct Update {
    pub location: Option<String>,
}

#[derive(Debug, Clone)]
struct Credentials {
    pub(crate) consumer_key: String,
    pub(crate) consumer_secret: String,
    pub(crate) token: String,
    pub(crate) token_secret: String,
}

impl Credentials {
    fn new() -> Result<Credentials, Box<dyn Error>> {
        let consumer_key = get_env_var("TWITTER_APP_API_KEY")?;
        let consumer_secret = get_env_var("TWITTER_APP_API_SECRET_KEY")?;
        let token = get_env_var("TWITTER_APP_ACCESS_TOKEN")?;
        let token_secret = get_env_var("TWITTER_APP_ACCESS_TOKEN_SECRET")?;

        Ok(Credentials {
            consumer_key,
            consumer_secret,
            token,
            token_secret,
        })
    }
}

pub(crate) trait FormField: Into<String> + Serialize + Clone {}
impl<T> FormField for T where T: Into<String> + Serialize + Clone {}

#[derive(Debug, Clone)]
pub(crate) struct TwitterClient {
    client: Client,
    credentials: Credentials,
    base_url: Url,
}

impl TwitterClient {
    pub(crate) fn new() -> Result<TwitterClient, Box<dyn Error>> {
        let client = Client::new();
        let credentials = Credentials::new()?;
        let base_url = "https://api.twitter.com".parse().ok().unwrap();

        Ok(TwitterClient {
            client,
            credentials,
            base_url,
        })
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
    pub(crate) async fn update_profile_location(
        &self,
        location: impl FormField,
    ) -> Result<Update, Box<dyn Error>> {
        let endpoint = "/1.1/account/update_profile.json";

        let url = self.base_url.join(endpoint).unwrap();

        let Credentials {
            consumer_key,
            consumer_secret,
            token,
            token_secret,
        } = &self.credentials;

        let secrets = Secrets::new(consumer_key, consumer_secret).token(token, token_secret);

        let client = self.client.clone();

        let response = client
            .oauth1(secrets)
            .request(Method::POST, url)
            .form(&[("location", location)]);

        let response = response.send().await?;

        let body = response.text().await?;
        let result: Result<Update, _> = from_str(&body);

        match result {
            Ok(update) => Ok(update),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TwitterClient;
    use httpmock::prelude::*;
    use serde_json::json;
    use std::error::Error;

    #[tokio::test]
    async fn test_update_twitter_profile_location() -> Result<(), Box<dyn Error>> {
        let mock_server = MockServer::start();
        let mock_location = "Kalamazoo, MI";
        let mock = mock_server.mock(|when, then| {
            when.method(POST)
                .path("/1.1/account/update_profile.json")
                .header("content-type", "application/x-www-form-urlencoded")
                .header_exists("authorization")
                .x_www_form_urlencoded_tuple("location", mock_location);
            then.status(200)
                .header("content-type", "application/json")
                .body(json!({ "location": mock_location }).to_string());
        });

        let mut client = TwitterClient::new()?;

        client.base_url = mock_server.base_url().parse()?;
        let result = client.update_profile_location(mock_location).await;

        mock.assert();
        assert!(result.is_ok(), "Result is not ok: {}", result.unwrap_err());

        assert_eq!(result.unwrap().location, Some(mock_location.to_string()),);

        Ok(())
    }
}
