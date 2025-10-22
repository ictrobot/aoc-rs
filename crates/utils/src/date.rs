//! Items for representing days, years and dates.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

/// Represents the [`Year`] and [`Day`] a puzzle was released.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Date {
    year: Year,
    day: Day,
}

impl Date {
    const FIRST_RELEASE_TIMESTAMP: u64 = 1_448_946_000; // 2015-12-01 05:00 UTC

    #[inline]
    #[must_use]
    pub const fn new(year: Year, day: Day) -> Option<Date> {
        if day.0 <= year.max_day().0 {
            Some(Self { year, day })
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub const fn year(self) -> Year {
        self.year
    }

    #[inline]
    #[must_use]
    pub const fn day(self) -> Day {
        self.day
    }

    fn release_timestamp(self) -> u64 {
        let mut days = u64::from(self.day.0) - 1;

        for year in 2016..=self.year.0 {
            let is_leap_year =
                (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
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

    /// The [`Date`] of the next puzzle.
    #[must_use]
    #[expect(clippy::cast_possible_truncation)]
    pub fn next_puzzle() -> Option<Date> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut date = Date {
            year: Year(2015),
            day: Day(1),
        };

        // Skip ahead whole years
        if now > Self::FIRST_RELEASE_TIMESTAMP {
            let year = 2015 + ((now - Self::FIRST_RELEASE_TIMESTAMP) / 60 / 60 / 24 / 366);
            if year > 9999 {
                return None;
            }
            date.year = Year(year as u16);
        }

        while date.release_timestamp() < now {
            if date.day.0 < date.year.max_day().0 {
                date.day.0 += 1;
            } else if date.year.0 < 9999 {
                date.year.0 += 1;
                date.day.0 = 1;
            } else {
                return None;
            }
        }

        Some(date)
    }
}

impl TryFrom<(Year, Day)> for Date {
    type Error = InvalidDateError;

    #[inline]
    fn try_from((year, day): (Year, Day)) -> Result<Self, Self::Error> {
        Self::new(year, day).ok_or(InvalidDateError(year))
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
    #[inline]
    #[must_use]
    pub const fn new(year: u16) -> Option<Self> {
        if year >= 2015 && year <= 9999 {
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
    /// ```compile_fail
    /// # use utils::date::Year;
    /// let year = Year::new_const::<2000>();
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn new_const<const YEAR: u16>() -> Self {
        const {
            assert!(YEAR >= 2015 && YEAR <= 9999);
        }
        Self(YEAR)
    }

    #[inline]
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        self.0
    }

    /// Returns the maximum day for the given year.
    ///
    /// # Examples
    /// ```
    /// # use utils::date::{Day, Year};
    /// assert_eq!(Year::new_const::<2015>().max_day(), Day::new_const::<25>());
    /// assert_eq!(Year::new_const::<2025>().max_day(), Day::new_const::<12>());
    /// ```
    #[inline]
    #[must_use]
    pub const fn max_day(self) -> Day {
        if self.0 < 2025 {
            Day::new_const::<25>()
        } else {
            Day::new_const::<12>()
        }
    }

    #[inline]
    pub fn days(self) -> impl Iterator<Item = Day> {
        (1..=self.max_day().to_u8()).map(|d| Day::new(d).unwrap())
    }
}

impl TryFrom<u16> for Year {
    type Error = InvalidYearError;

    #[inline]
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
    #[inline]
    #[must_use]
    pub const fn new(day: u8) -> Option<Self> {
        if day >= 1 && day <= 25 {
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
    /// ```compile_fail
    /// # use utils::date::Day;
    /// let year = Day::new_const::<26>();
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn new_const<const DAY: u8>() -> Self {
        const {
            assert!(DAY >= 1 && DAY <= 25);
        }
        Self(DAY)
    }

    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Day {
    type Error = InvalidDayError;

    #[inline]
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

/// Error type returned when trying to construct an invalid [`Date`].
#[derive(Debug)]
pub struct InvalidDateError(Year);

impl Display for InvalidDateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid day for year {:#}", self.0)
    }
}

impl Error for InvalidDateError {}

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
