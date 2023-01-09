#![allow(dead_code)]

use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone)]
pub struct Meter {
    current: Option<f64>,
    max: f64,
    length: u8,
    meter: String,
}

impl Meter {
    pub fn new<T: Into<f64>>(current: Option<T>) -> Self {
        let current = current.map(|current| current.into());
        let max = 170f64;
        let length = 8;
        let meter = Self::create_meter(current, max, length).unwrap_or_else(|_| {
            panic!("Failed to create meter. Current value: {:?}", current);
        });
        Self {
            current,
            max,
            length,
            meter,
        }
    }

    /// Generates a meter with emoji to show how close you are to burnout.
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
    /// let meter = Meter::new(1f64);
    /// assert_eq!(meter, "🟩⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️");
    /// ```
    pub fn build(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        self.meter = Self::create_meter(self.current, self.max, self.length)?;
        Ok(self)
    }

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
    /// let meter = create_meter(Some(10f64), 100f64, 10)?;
    /// assert_eq!(meter, "🟩⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️");
    /// ```
    fn create_meter(current: Option<f64>, max: f64, length: u8) -> Result<String, Box<dyn Error>> {
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

        // make sure the meter is never longer than the length set
        if filled > length {
            filled = length;
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

    pub fn current<T: Into<f64>>(&mut self, current: T) -> &mut Self {
        self.current = Some(current.into());
        self
    }

    pub fn max<T: Into<f64>>(&mut self, max: T) -> &mut Self {
        self.max = max.into();
        self
    }

    pub fn length<T: Into<u8>>(&mut self, length: T) -> &mut Self {
        self.length = length.into();
        self
    }
}

impl Display for Meter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let meter = &self.meter;
        write!(f, "{}", meter)
    }
}

// allows comparing meter to a String or &str
impl PartialEq<String> for Meter {
    fn eq(&self, other: &String) -> bool {
        self.meter == *other
    }
}

impl PartialEq<&str> for Meter {
    fn eq(&self, other: &&str) -> bool {
        self.meter == *other
    }
}

impl PartialEq<str> for Meter {
    fn eq(&self, other: &str) -> bool {
        self.meter == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_meter() -> Result<(), Box<dyn Error>> {
        let mut meter = Meter::new(None as Option<f64>);

        meter.current(0).build()?;
        assert_eq!(meter, "⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️");

        meter.current(1).length(10).max(10).build()?;
        assert_eq!(meter, "🟩⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️⬜️");

        meter.current(5.5).build()?;
        assert_eq!(meter, "🟨🟨🟨🟨🟨⬜️⬜️⬜️⬜️⬜️");

        meter.current(7.1).build()?;
        assert_eq!(meter, "🟧🟧🟧🟧🟧🟧🟧⬜️⬜️⬜️");

        meter.current(9.4).build()?;
        assert_eq!(meter, "🟥🟥🟥🟥🟥🟥🟥🟥🟥⬜️");

        meter.length(5).build()?;
        assert_eq!(meter, "🟥🟥🟥🟥⬜️");

        meter.length(20).build()?;
        assert_eq!(meter, "🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥🟥⬜️⬜️");

        Ok(())
    }
}
