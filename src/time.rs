use serde::{Deserialize, Deserializer, Serializer, de::Error};
use chrono::*;

#[cfg(feature = "client")]
use crate::{RemindmeError, RemindmeResult};

pub type Time = DateTime<Local>;

pub fn now() -> Time {
    Local::now()
}

pub fn serialize<S>(t: &Time, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&t.to_rfc3339())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    DateTime::<FixedOffset>::parse_from_rfc3339(s)
        .map(|dt| dt.into())
        .map_err(D::Error::custom)
}

#[cfg(feature = "client")]
pub fn parse(s: &str) -> RemindmeResult<Time> {
    if s.ends_with("s") {
        let seconds: u64 = s[0..s.len() - 1].parse()
            .map_err(|_| RemindmeError::ParseTime(s.to_string()))?;
        let delta = TimeDelta::new(seconds as i64, 0)  // TODO check safety of conversion
            .ok_or(RemindmeError::TimeDeltaSec(seconds))?;
        now().checked_add_signed(delta)
            .ok_or(RemindmeError::TimeDeltaSec(seconds))
    } else if s.ends_with("m") {
        let minutes: u64 = s[0..s.len() - 1].parse()
            .map_err(|_| RemindmeError::ParseTime(s.to_string()))?;
        let seconds = minutes * 60;
        let delta = TimeDelta::new(seconds as i64, 0)  // TODO check safety of conversion
            .ok_or(RemindmeError::TimeDeltaMin(minutes))?;
        now().checked_add_signed(delta)
            .ok_or(RemindmeError::TimeDeltaMin(minutes))
    } else if s.ends_with("h") {
        let hours: u64 = s[0..s.len() - 1].parse()
            .map_err(|_| RemindmeError::ParseTime(s.to_string()))?;
        let seconds = hours * 60 * 60;
        let delta = TimeDelta::new(seconds as i64, 0)  // TODO check safety of conversion
            .ok_or(RemindmeError::TimeDeltaH(hours))?;
        now().checked_add_signed(delta)
            .ok_or(RemindmeError::TimeDeltaH(hours))
    } else if s.ends_with("d") {
        let days: u64 = s[0..s.len() - 1].parse()
            .map_err(|_| RemindmeError::ParseTime(s.to_string()))?;
        let seconds = days * 60 * 60 * 24;
        let delta = TimeDelta::new(seconds as i64, 0)  // TODO check safety of conversion
            .ok_or(RemindmeError::TimeDeltaD(days))?;
        now().checked_add_signed(delta)
            .ok_or(RemindmeError::TimeDeltaD(days))
    } else if s.ends_with("w") {
        let weeks: u64 = s[0..s.len() - 1].parse()
            .map_err(|_| RemindmeError::ParseTime(s.to_string()))?;
        let seconds = weeks * 60 * 60 * 24 * 7;
        let delta = TimeDelta::new(seconds as i64, 0)  // TODO check safety of conversion
            .ok_or(RemindmeError::TimeDeltaW(weeks))?;
        now().checked_add_signed(delta)
            .ok_or(RemindmeError::TimeDeltaW(weeks))
    } else {
        NaiveDateTime::parse_from_str(s, "%d.%m.%Y %H:%M")
            .or(NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M"))
            .map_err(|_| RemindmeError::ParseTime(s.to_string()))
            .map(|dt| dt.and_utc().into())
    }
}

