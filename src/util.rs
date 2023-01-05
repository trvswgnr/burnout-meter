use std::error::Error;

/// Create a meter with emoji to show how close you are to burnout.
///
/// # Errors
/// Returns an error if the current value is `None`.
///
/// # Examples
///
/// ```
/// use util::create_meter;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let meter = create_meter(Some(10f64), 170f64, 8)?;
///     assert_eq!(meter, "🟩🟩🟩🟩🟩🟩🟩🟩");
///     Ok(())
/// }
/// ```
pub(crate) fn create_meter(
    current: Option<f64>,
    max: f64,
    length: u8,
) -> Result<String, Box<dyn Error>> {
    if current.is_none() {
        return Err("No current value".into());
    }

    let current = current.unwrap();

    // get the current without the decimal
    let hours = current.floor();
    let remainder = current - hours;
    let percentage = current / max;
    let mut filled = (percentage * length as f64).floor() as u8;

    if remainder > 0.5 && length >= 10 && filled < length {
        filled += 1;
    }

    let empty = length - filled;

    let mut emoji = String::from("🟩");

    if percentage > 0.45 {
        emoji = String::from("🟨");
    }

    if percentage > 0.7 {
        emoji = String::from("🟧");
    }

    if percentage > 0.94 {
        emoji = String::from("🟥");
    }

    let blank = "⬜️";

    let meter = emoji.repeat(filled as usize) + &blank.repeat(empty as usize);

    Ok(meter)
}

#[cfg(test)]
mod tests {
    use super::*;

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
