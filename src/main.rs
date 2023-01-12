//! Update your Twitter profile location with a meter of how close you are to burnout.
//!
//! Uses the [WakaTime API] to get the total time spent coding in the last 30 days.
//! Then creates a meter with emoji to show how close you are to burnout.
//! Finally, updates your Twitter profile location with the meter.
//!
//! [WakaTime API]: https://wakatime.com/developers

mod app;
mod meter;
mod twitter;
mod util;
mod wakatime;

use app::{App, AppSettings};
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = AppSettings::default();
    let mut app = App::new(settings)?;
    app.run().await?;
    Ok(())
}
