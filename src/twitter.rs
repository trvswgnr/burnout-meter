use reqwest::{Client, Method, Url};
use reqwest_oauth1::OAuthClientProvider;
use reqwest_oauth1::Secrets;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::error::Error;
#[derive(Deserialize, Debug, Serialize)]
pub struct Profile {
    pub location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

pub trait FormField: Into<String> + Serialize + Clone {}
impl<T> FormField for T where T: Into<String> + Serialize + Clone {}

#[derive(Debug, Clone)]
pub struct Twitter {
    client: Client,
    credentials: Credentials,
    base_url: Url,
}

impl Twitter {
    pub fn new(credentials: Credentials) -> Result<Self, Box<dyn Error>> {
        let client = Client::new();
        let base_url = "https://api.twitter.com".parse().ok().unwrap();

        Ok(Self {
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
    pub async fn update_location(
        &self,
        location: impl FormField,
    ) -> Result<Profile, Box<dyn Error>> {
        let endpoint = "/1.1/account/update_profile.json";

        let url = self.base_url.join(endpoint).unwrap();

        let secrets = Secrets::new(
            &self.credentials.consumer_key,
            &self.credentials.consumer_secret,
        )
        .token(
            &self.credentials.access_token,
            &self.credentials.access_token_secret,
        );

        let client = self.client.clone();

        let response = client
            .oauth1(secrets)
            .request(Method::POST, url)
            .form(&[("location", location)]);

        let response = response.send().await?;

        let body = response.text().await?;
        let result: Result<Profile, _> = from_str(&body);

        match result {
            Ok(update) => Ok(update),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Credentials, Twitter};
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

        let credentials = Credentials {
            consumer_key: "consumer_key".to_string(),
            consumer_secret: "consumer_secret".to_string(),
            access_token: "access_token".to_string(),
            access_token_secret: "access_token_secret".to_string(),
        };

        let mut client = Twitter::new(credentials)?;

        client.base_url = mock_server.base_url().parse()?;
        let result = client.update_location(mock_location).await;

        mock.assert();
        assert!(result.is_ok(), "Result is not ok: {}", result.unwrap_err());

        assert_eq!(result.unwrap().location, Some(mock_location.to_string()),);

        Ok(())
    }
}
