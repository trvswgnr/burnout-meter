//! Update your Twitter profile location with a meter of how close you are to burnout.
//!
//! Uses the [WakaTime API] to get the total time spent coding in the last 30 days.
//! Then creates a meter with emoji to show how close you are to burnout.
//! Finally, updates your Twitter profile location with the meter.
//!
//! [WakaTime API]: https://wakatime.com/developers

#[macro_use]
extern crate dotenv_codegen;

use std::error::Error;

mod wakatime;

mod twitter;

mod util;

struct CollectedData {
    update: twitter::Update,
    hours: Option<f64>,
    meter: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let data = run().await?;

    match data.update.location {
        Some(location) => println!("Location updated to {location}"),
        None => panic!("Location not updated"),
    }

    let hours = data.hours.unwrap_or(0f64);
    let hours_til_burnout = ((170f64 - hours) * 100f64).round() / 100f64;

    println!("{hours} hours in the last 30 days");
    println!("Hours til burnout: {hours_til_burnout}",);
    println!("Meter: {}", data.meter);

    Ok(())
}

async fn run() -> Result<CollectedData, Box<dyn Error>> {
    let hours = wakatime::get_time_last_n_days(30, None).await?;
    let meter = util::create_meter(hours, 170f64, 8)?;
    let location = format!("{meter} to burnout");
    let updated = twitter::update_twitter_profile_location(&location, None).await?;

    Ok(CollectedData {
        update: updated,
        hours,
        meter,
    })
}
