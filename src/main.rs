//! Update your Twitter profile location with a meter of how close you are to burnout.
//!
//! Uses the [WakaTime API] to get the total time spent coding in the last 30 days.
//! Then creates a meter with emoji to show how close you are to burnout.
//! Finally, updates your Twitter profile location with the meter.
//!
//! [WakaTime API]: https://wakatime.com/developers

mod meter;
mod twitter;
mod util;
mod wakatime;

use std::error::Error;
use twitter::Twitter;
use wakatime::WakaTime;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let wakatime = WakaTime::new()?;
    let twitter = Twitter::new()?;

    let hours = match wakatime.get_time_last_n_days(30).await {
        Ok(hours) => hours.unwrap(),
        Err(_) => panic!("Failed to get hours from WakaTime"),
    };

    let burnout_meter = meter::Builder::new().current(hours).build()?.to_string();
    let location = format!("{} to burnout", burnout_meter);
    let profile = twitter.update_profile_location(location).await?;

    match profile.location {
        Some(location) => println!("Location updated to {location}"),
        None => panic!("Location not updated"),
    };

    let hours_til_burnout = ((170f64 - hours) * 100f64).round() / 100f64;

    println!("{hours} hours in the last 30 days");
    println!("Hours til burnout: {hours_til_burnout}",);
    println!("Generated Meter: {}", burnout_meter);

    Ok(())
}
