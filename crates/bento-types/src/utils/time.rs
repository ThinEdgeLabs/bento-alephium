use chrono::{DateTime, NaiveDateTime, Utc};

// Convert timestamp in milliseconds to NaiveDateTime
pub fn timestamp_millis_to_naive_datetime(timestamp_millis: i64) -> NaiveDateTime {
    let seconds = timestamp_millis / 1000;
    let nanoseconds = (timestamp_millis % 1000) * 1_000_000;
    DateTime::<Utc>::from_timestamp(seconds, nanoseconds as u32)
        .map(|datetime_utc| datetime_utc.naive_utc())
        .unwrap_or_default()
}
