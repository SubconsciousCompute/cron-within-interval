//! cron-office
//!
//! Extended cron shorthands for Office and WFH folks.
//!
//! In addition to standard expression supported by excellent crate cron, we support following type
//! of expressions.
//!
//! - `@daily{H=9-17}` means run once between 9am and 5pm chosen randomly.
//! - `@daily{H=9-12,H=15-20}` means run once between 9am and 12pm or between 3pm and 8pm.
//!
//! Similarly one can pass daily contraints to @weekly.
//!
//! - `@weekly{D=1-5}` mean  run once per week between day 1 and day 5.
//! - `@weekly{D=1-5,H=9-12}` run once per week between day 1 and day 5 and between 9am and 12pm.
//! - `@weekly{H=9-12}` run once per week at any day chosen randomly and between 9am and 12pm.

use std::collections::HashMap;
use std::str::FromStr;

use cron::Schedule;
use regex::Regex;

lazy_static::lazy_static! {
    static ref RE: Regex = Regex::new(
        r"(?x)
    (?P<shorthand>\@(daily|weekly|monthly|yearly))\{ 
    (?P<constraints> 
        (h|d|w|m)\=(\d+\-\d+)(\,(h|d|w|m)\=\d+\-\d+)*
    )
    \}"
    )
    .unwrap();
}

/// Wrapper around cron::Schedule
#[derive(Debug)]
pub struct CronOffice {
    inner: cron::Schedule,
    /// Hourly constraints
    constraints: HashMap<String, Vec<Interval>>,
}

impl FromStr for CronOffice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let caps = RE.captures(s).expect("valid pattern");
        let shorthand = &caps["shorthand"];
        let cs = &caps["constraints"];

        let inner = Schedule::from_str(&shorthand)?;
        let mut constraints = HashMap::new();

        for constraint in cs.split(',') {
            let mut key_with_interval = constraint.splitn(2, '=');
            let key = key_with_interval.next().expect("timescale key");
            let interval: Interval = key_with_interval
                .next()
                .expect("interval")
                .parse()
                .expect("valid interval");

            constraints
                .entry(key.to_string())
                .or_insert(vec![])
                .push(interval);
        }

        Ok(Self { inner, constraints })
    }
}

impl CronOffice {
    pub fn upcoming<Z>(&self, timezone: Z) -> cron::ScheduleIterator<'_, Z>
    where
        Z: chrono::TimeZone,
    {
        self.inner.upcoming(timezone).map(|sch| sch)
    }
}

/// Interval (low, high)
#[derive(Default, Debug)]
struct Interval(i16, i16);

impl FromStr for Interval {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut low_high = s.splitn(2, '-');
        let low = low_high.next().expect("low part").parse::<i16>()?;
        let high = low_high.next().expect("high part").parse::<i16>()?;
        if high <= low {
            anyhow::bail!("Invalid interval: High {high} is less than low {low}");
        }
        Ok(Self(low, high))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_cron_office() {
        let sch = CronOffice::from_str("@daily{h=9-17,h=21-23}").unwrap();
        println!("{sch:?}");

        let mut schedules = vec![];
        for datetime in sch.upcoming(Utc).take(10) {
            schedules.push(datetime);
            println!("-> {datetime:?}");
        }
    }

    #[test]
    fn test_cron_sanity() {
        let sch = Schedule::from_str("@daily").unwrap();
        let mut schedules = vec![];
        for datetime in sch.upcoming(Utc).take(10) {
            schedules.push(datetime);
            println!("-> {datetime:?}");
        }
        assert_eq!(schedules.len(), 10);
        for i in 1..schedules.len() {
            let diff = schedules[i] - schedules[i - 1];
            assert_eq!(diff.num_hours(), 24);
        }
    }
}
