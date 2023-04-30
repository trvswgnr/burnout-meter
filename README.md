\* This now only works if you have a paid subscription to the Twitter API, which doesn't currently offer a reasonably priced tier for hobbyists. Unfortunately, I can't recommend using this anymore.

# WakaTime Code Burnout Meter in Twitter Profile

![tests](https://github.com/trvswgnr/burnout-meter/actions/workflows/test.yml/badge.svg?branch=main)
![last run](https://github.com/trvswgnr/burnout-meter/actions/workflows/run.yml/badge.svg?branch=main&event=schedule)

Inspired by [trash's Twitter profile](https://twitter.com/trashh_dev), this repo adds a burnout meter to your Twitter profile. It uses  [WakaTime](https://wakatime.com/) to get your coding activity and calculates the burnout percentage based on the number of hours you've coded in the last 30 days. It then creates a meter with emoji and updates your Twitter profile location with the meter.

Here's how it will look at the different stages of burnout:

🟩🟩⬜️⬜️⬜️⬜️⬜️⬜️ to burnout

🟨🟨🟨🟨⬜️⬜️⬜️⬜️ to burnout

🟧🟧🟧🟧🟧🟧⬜️⬜️ to burnout

🟥🟥🟥🟥🟥🟥🟥🟥 to burnout

It uses cron with GitHub Actions to run every 6 hours, but can also be run manually.

## Usage

1. Fork or Clone this repository
1. Get your [WakaTime API key](https://wakatime.com/settings/account)
1. Create a [Twitter app](https://developer.twitter.com/en/apps) and get your API keys
1. Create a [GitHub secret](https://docs.github.com/en/actions/reference/encrypted-secrets) for each of the following:
    - `WAKATIME_API_KEY` - Your WakaTime API key
    - `TWITTER_ACCESS_TOKEN` - Access token for your Twitter app
    - `TWITTER_ACCESS_TOKEN_SECRET` - Access token secret for your Twitter app
    - `TWITTER_CONSUMER_KEY` - Consumer API key for your Twitter app
    - `TWITTER_CONSUMER_SECRET` - Consumer API secret key for your Twitter app
1. You can also optionaly set the following secrets to customize the meter:
    - `TIMEZONE_OFFSET` - The timezone offset in hours (default: 0 for UTC)
    - `BURNOUT_LIMIT` - The number of hours before you're to be considered "burned out" (default: 40)
    - `METER_LENGTH` - The number of emoji to use for the meter (default: 8)

## License

[MIT](LICENSE)

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

---

Created with [💜](https://travisaw.com) and [🦀](https://www.rust-lang.org/) by [Travis Aaron Wagner](https://github.com/trvswgnr)
