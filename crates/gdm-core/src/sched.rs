// SPDX-License-Identifier: RPL-1.5
//! Queue scheduler times (F08, Q-03). A schedule is local wall-clock time + a rule (days of the week, or one
//! date), never a stored UTC instant, so a DST change or a new time zone moves it with the user's clock.
//! Nonexistent local time (spring forward) => the first valid minute after it; a repeated time (fall back) =>
//! the earlier instant, and the caller runs a date key once (idempotent).
use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Schedule {
    pub enabled: bool,
    /// "HH:MM" local time; None = no timed start (start by hand)
    pub start: Option<String>,
    /// "HH:MM" local time to stop the queue (may be after midnight: earlier than `start` = next day)
    pub stop: Option<String>,
    /// Monday = bit 0 .. Sunday = bit 6; 0 = once, on `date`
    pub days: u8,
    /// "YYYY-MM-DD" for a one-time start
    pub date: Option<String>,
    /// periodic synchronisation: start again N minutes after the previous start (IDM "every X hours Y min")
    pub every_min: Option<u32>,
}

pub fn hm(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").ok()
}

/// local date + time => the instant it means in `tz`: gap => first valid minute after it; fold => earlier
pub fn resolve<Tz: TimeZone>(tz: &Tz, local: NaiveDateTime) -> Option<DateTime<Utc>> {
    for m in 0..=180 {
        match tz.from_local_datetime(&(local + Duration::minutes(m))) {
            LocalResult::Single(t) => return Some(t.with_timezone(&Utc)),
            LocalResult::Ambiguous(a, b) => return Some(a.with_timezone(&Utc).min(b.with_timezone(&Utc))),
            LocalResult::None => continue,
        }
    }
    None
}

impl Schedule {
    fn day_ok(&self, d: NaiveDate) -> bool {
        if self.days == 0 {
            return self.date.as_deref().and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()) == Some(d);
        }
        self.days & (1 << d.weekday().num_days_from_monday()) != 0
    }

    /// the next timed start strictly after `after` whose date key (local "YYYY-MM-DD") is not `last_key`
    /// (Q-03: a key runs once even when the wall clock repeats an hour)
    pub fn next_start<Tz: TimeZone>(&self, tz: &Tz, after: DateTime<Utc>, last_key: Option<&str>) -> Option<(DateTime<Utc>, String)> {
        if !self.enabled {
            return None;
        }
        let t = hm(self.start.as_deref()?)?;
        let today = after.with_timezone(tz).date_naive();
        for i in -1..=8 {
            let d = today + Duration::days(i);
            let key = d.format("%Y-%m-%d").to_string();
            if !self.day_ok(d) || last_key.is_some_and(|k| k >= key.as_str()) {
                continue;
            }
            if let Some(at) = resolve(tz, d.and_time(t)) {
                if at > after {
                    return Some((at, key));
                }
            }
        }
        None
    }

    /// the stop instant for a run started for date key `key`
    pub fn stop_at<Tz: TimeZone>(&self, tz: &Tz, key: &str) -> Option<DateTime<Utc>> {
        let stop = hm(self.stop.as_deref()?)?;
        let d = NaiveDate::parse_from_str(key, "%Y-%m-%d").ok()?;
        let start = self.start.as_deref().and_then(hm).unwrap_or(NaiveTime::MIN);
        let d = if stop <= start { d + Duration::days(1) } else { d };
        resolve(tz, d.and_time(stop))
    }

    /// should a started queue still be running at `now` (the stop time, if any, has not passed)?
    pub fn within<Tz: TimeZone>(&self, tz: &Tz, key: &str, now: DateTime<Utc>) -> bool {
        self.stop_at(tz, key).is_none_or(|s| now < s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::America::New_York;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }
    fn every_day(start: &str) -> Schedule {
        Schedule { enabled: true, start: Some(start.into()), days: 0x7f, ..Default::default() }
    }

    /// Q-03: America/New_York 2026-03-08 has no 02:30 (spring forward 02:00 -> 03:00)
    #[test]
    fn q03_spring_forward_runs_at_first_valid_time_once() {
        let s = every_day("02:30");
        let (at, key) = s.next_start(&New_York, utc("2026-03-08T05:00:00Z"), None).unwrap(); // 00:00 EST
        assert_eq!(key, "2026-03-08");
        assert_eq!(at, utc("2026-03-08T07:00:00Z"), "03:00 EDT, the first valid minute after the gap");
        // after it ran: the same key is never picked again; next is 03-09 02:30 EDT
        let (at2, key2) = s.next_start(&New_York, at, Some(&key)).unwrap();
        assert_eq!((at2, key2.as_str()), (utc("2026-03-09T06:30:00Z"), "2026-03-09"));
    }

    /// Q-03: 2026-11-01 01:30 happens twice (EDT then EST): run once, at the earlier one
    #[test]
    fn q03_fall_back_runs_once() {
        let s = every_day("01:30");
        let (at, key) = s.next_start(&New_York, utc("2026-11-01T04:00:00Z"), None).unwrap(); // 00:00 EDT
        assert_eq!(at, utc("2026-11-01T05:30:00Z"), "01:30 EDT");
        // one hour later the wall clock shows 01:30 again (EST): the key already ran => tomorrow
        let (_, key2) = s.next_start(&New_York, utc("2026-11-01T06:29:00Z"), Some(&key)).unwrap();
        assert_eq!(key2, "2026-11-02");
    }

    #[test]
    fn days_once_and_stop_after_midnight() {
        let mon_wed = Schedule { enabled: true, start: Some("23:00".into()), stop: Some("06:00".into()), days: 0b101, ..Default::default() };
        // 2026-11-10 is a Tuesday => next is Wednesday 11
        let (at, key) = mon_wed.next_start(&Utc, utc("2026-11-10T12:00:00Z"), None).unwrap();
        assert_eq!((at, key.as_str()), (utc("2026-11-11T23:00:00Z"), "2026-11-11"));
        assert_eq!(mon_wed.stop_at(&Utc, &key), Some(utc("2026-11-12T06:00:00Z")));
        assert!(mon_wed.within(&Utc, &key, utc("2026-11-12T05:59:00Z")));
        assert!(!mon_wed.within(&Utc, &key, utc("2026-11-12T06:00:00Z")));
        let once = Schedule { enabled: true, start: Some("08:00".into()), date: Some("2026-12-01".into()), ..Default::default() };
        assert_eq!(once.next_start(&Utc, utc("2026-11-10T00:00:00Z"), None), None, "more than 8 days ahead: not yet");
        assert!(once.next_start(&Utc, utc("2026-11-28T00:00:00Z"), None).is_some());
        assert!(once.next_start(&Utc, utc("2026-12-01T09:00:00Z"), None).is_none(), "passed");
        assert!(Schedule { enabled: false, ..every_day("01:00") }.next_start(&Utc, utc("2026-11-10T00:00:00Z"), None).is_none());
    }
}
