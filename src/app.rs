use crate::{
    meter,
    twitter::{self, Twitter},
    util::get_env_var,
    wakatime::WakaTime,
};
use std::error::Error;

pub struct App {
    wakatime: WakaTime,
    twitter: Twitter,
    burnout_meter: meter::Builder,
    settings: AppSettings,
}

impl App {
    pub fn new(settings: AppSettings) -> Result<Self, Box<dyn Error>> {
        return Ok(Self {
            wakatime: WakaTime::new(settings.wakatime_api_key())?,
            twitter: Twitter::new(settings.twitter_credentials())?,
            burnout_meter: meter::Builder::new(),
            settings,
        });
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let hours = match self
            .wakatime
            .get_time_last_n_days(self.settings.burnout_days())
            .await
        {
            Ok(hours) => hours.unwrap(),
            Err(_) => panic!("Failed to get hours from WakaTime"),
        };

        self.burnout_meter
            .set_max(self.settings.burnout_limit())
            .set_length(self.settings.meter_length())
            .set_current(hours)
            .build()?;

        let location = format!("{} to burnout", self.burnout_meter);
        let profile = self.twitter.update_location(location).await?;

        match profile.location {
            Some(location) => println!("Location updated to {location}"),
            None => panic!("Location not updated"),
        };

        let hours_til_burnout = ((self.burnout_meter.max() - hours) * 100f64).round() / 100f64;

        println!("{hours} hours in the last 30 days");
        println!("Hours til burnout: {hours_til_burnout}",);
        println!("Generated Meter: {}", self.burnout_meter);

        Ok(())
    }
}

pub struct AppSettings {
    wakatime_api_key: String,
    twitter_credentials: twitter::Credentials,
    burnout_limit: f64,
    burnout_days: i64,
    meter_length: u8,
}

impl AppSettings {
    fn wakatime_api_key(&self) -> &str {
        self.wakatime_api_key.as_ref()
    }

    fn twitter_credentials(&self) -> twitter::Credentials {
        self.twitter_credentials.clone()
    }

    fn burnout_limit(&self) -> f64 {
        self.burnout_limit
    }

    fn burnout_days(&self) -> i64 {
        self.burnout_days
    }

    fn meter_length(&self) -> u8 {
        self.meter_length
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            wakatime_api_key: get_env_var("WAKATIME_API_KEY").unwrap(),
            twitter_credentials: twitter::Credentials {
                consumer_key: get_env_var("TWITTER_CONSUMER_KEY").unwrap(),
                consumer_secret: get_env_var("TWITTER_CONSUMER_SECRET").unwrap(),
                access_token: get_env_var("TWITTER_ACCESS_TOKEN").unwrap(),
                access_token_secret: get_env_var("TWITTER_ACCESS_TOKEN_SECRET").unwrap(),
            },
            burnout_limit: get_env_var("BURNOUT_LIMIT").unwrap_or(180.0),
            burnout_days: get_env_var("BURNOUT_DAYS").unwrap_or(30),
            meter_length: get_env_var("METER_LENGTH").unwrap_or(8),
        }
    }
}
