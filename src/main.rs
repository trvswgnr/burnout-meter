//! Update your Twitter profile location with a meter of how close you are to burnout.
//!
//! Uses the [WakaTime API] to get the total time spent coding in the last 30 days.
//! Then creates a meter with emoji to show how close you are to burnout.
//! Finally, updates your Twitter profile location with the meter.
//!
//! [WakaTime API]: https://wakatime.com/developers

mod twitter;
mod util;
mod wakatime;

use std::error::Error;
use twitter::Twitter;
use util::create_meter;

use crate::wakatime::WakaTime;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let wakatime = WakaTime::new()?;
    let twitter = Twitter::new()?;
    let hours = wakatime.get_time_last_n_days(30).await?;
    let meter = create_meter(hours, 170f64, 8)?;
    let location = meter.clone() + " to burnout";
    let update = twitter.update_profile_location(location).await?;

    match update.location {
        Some(location) => println!("Location updated to {location}"),
        None => panic!("Location not updated"),
    };

    let hours = hours.unwrap_or(0f64);
    let hours_til_burnout = ((170f64 - hours) * 100f64).round() / 100f64;

    println!("{hours} hours in the last 30 days");
    println!("Hours til burnout: {hours_til_burnout}",);
    println!("Generated Meter: {}", meter);

    Ok(())
}
