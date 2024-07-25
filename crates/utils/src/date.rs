//! Items for representing days, years and dates.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

/// Represents the [`Year`] and [`Day`] a puzzle was released.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Date {
    pub year: Year,
    pub day: Day,
}

impl Date {
    const FIRST_RELEASE_TIMESTAMP: u64 = 1_448_946_000; // 2015-12-01 05:00 UTC

    fn release_timestamp(self) -> u64 {
        let mut days = u64::from(self.day.0) - 1;

        for year in 2016..=self.year.0 {
            let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            days += if is_leap_year { 366 } else { 365 };
        }

        Self::FIRST_RELEASE_TIMESTAMP + (days * 86400)
    }

    /// The [`SystemTime`] when the puzzle was released/is scheduled to release.
    ///
    /// This can be compared to [`SystemTime::now()`] to check if a puzzle is released, or how long
    /// remains until release.
    #[must_use]
    pub fn release_time(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.release_timestamp())
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} day {}", self.year.0, self.day.0)
    }
}

/// Represents a 4-digit year, 2015 or later.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Year(u16);

impl Year {
    #[must_use]
    pub fn new(year: u16) -> Option<Self> {
        if (2015..=9999).contains(&year) {
            Some(Self(year))
        } else {
            None
        }
    }

    /// # Panics
    ///
    /// Panics at compile time (enforced by the use of const generics) if the year is out of range
    ///
    /// # Examples
    ///
    /// ```
    /// # use utils::date::Year;
    /// let year = Year::new_const::<2015>();
    /// ```
    ///
    /// ```should_panic
    /// # use utils::date::Year;
    /// let year = Year::new_const::<2000>();
    /// ```
    #[must_use]
    pub const fn new_const<const YEAR: u16>() -> Self {
        assert!(YEAR >= 2015 && YEAR <= 9999);
        Self(YEAR)
    }

    #[must_use]
    pub const fn to_u16(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for Year {
    type Error = InvalidYearError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(InvalidYearError)
    }
}

impl FromStr for Year {
    type Err = InvalidYearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = s.parse::<u16>() {
            v.try_into()
        } else {
            Err(InvalidYearError)
        }
    }
}

impl Display for Year {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.0)
        } else {
            write!(f, "Year {}", self.0)
        }
    }
}

/// Represents a day between 1 and 25 (inclusive).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Day(u8);

impl Day {
    #[must_use]
    pub fn new(day: u8) -> Option<Self> {
        if (1..=25).contains(&day) {
            Some(Self(day))
        } else {
            None
        }
    }

    /// # Panics
    ///
    /// Panics at compile time (enforced by the use of const generics) if the day is out of range
    ///
    /// # Examples
    ///
    /// ```
    /// # use utils::date::Day;
    /// let year = Day::new_const::<17>();
    /// ```
    ///
    /// ```should_panic
    /// # use utils::date::Day;
    /// let year = Day::new_const::<26>();
    /// ```
    #[must_use]
    pub const fn new_const<const DAY: u8>() -> Self {
        assert!(DAY >= 1 && DAY <= 25);
        Self(DAY)
    }

    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Day {
    type Error = InvalidDayError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(InvalidDayError)
    }
}

impl FromStr for Day {
    type Err = InvalidDayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = s.parse::<u8>() {
            v.try_into()
        } else {
            Err(InvalidDayError)
        }
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:02}", self.0)
        } else {
            write!(f, "Day {:02}", self.0)
        }
    }
}

/// Error type returned when trying to convert an invalid value to a [`Year`].
#[derive(Debug)]
pub struct InvalidYearError;

impl Display for InvalidYearError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("invalid year")
    }
}

impl Error for InvalidYearError {}

/// Error type returned when trying to convert an invalid value to a [`Day`].
#[derive(Debug)]
pub struct InvalidDayError;

impl Display for InvalidDayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("invalid day")
    }
}

impl Error for InvalidDayError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_release_timestamps() {
        assert_eq!(
            Date {
                year: Year(2015),
                day: Day(1)
            }
            .release_timestamp(),
            1_448_946_000
        );
        assert_eq!(
            Date {
                year: Year(2016),
                day: Day(23)
            }
            .release_timestamp(),
            1_482_469_200
        );
        assert_eq!(
            Date {
                year: Year(2017),
                day: Day(11)
            }
            .release_timestamp(),
            1_512_968_400
        );
        assert_eq!(
            Date {
                year: Year(2021),
                day: Day(2)
            }
            .release_timestamp(),
            1_638_421_200
        );
        assert_eq!(
            Date {
                year: Year(2023),
                day: Day(7)
            }
            .release_timestamp(),
            1_701_925_200
        );
    }
}
