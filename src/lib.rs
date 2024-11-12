//! cron-within-interval
//!
//! Extended cron shorthands that support sampling from given interval. In addition to standard
//! expression supported by excellent crate cron, we support following type of expressions.
//!
//! The random number is seeded so you always get the same sequence.
//!
//! - `@daily{h=9-17}` means run once between 9am and 5pm chosen randomly.  
//! - `@daily{h=9-12,h=15-20}` means run once between 9am and 12pm or between 3pm and 8pm.
//!
//! Similarly one can pass daily contraints to @weekly.
//!
//! - `@weekly{d=1-5}` mean  run once per week between day 1 and day 5.  
//! - `@weekly{d=1-5,h=9-12}` run once per week between day 1 and day 5 and between 9am
//!    and 12pm.  
//! - `@weekly{h=9-12}` run once per week at any day chosen randomly and between 9am
//!    and 12pm.

use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;

use cron::Schedule;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use regex::Regex;

/// Global seed fro rngs.
const SEED: u64 = 1443;

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

thread_local! {
    static RNG: RefCell<ChaCha8Rng> = RefCell::new(ChaCha8Rng::seed_from_u64(SEED));
}

/// Wrapper around cron::Schedule
#[derive(Debug)]
pub struct CronWithRandomness {
    /// inner cron schedule
    pub schedule: cron::Schedule,
    /// Constraints
    constraints: HashMap<String, Vec<Interval>>,
}

impl FromStr for CronWithRandomness {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if !s.contains('{') {
            // this is standard cron expression. It may have 5 fields (linux, missing seconds). Add
            // second to 0 and continue.
            let num_fields = s.trim().split(' ').collect::<Vec<_>>().len();
            let expr = if num_fields == 5 {
                format!("0 {s}") // prepend seconds
            } else if num_fields == 6 {
                s.to_string()
            } else {
                anyhow::bail!("expression must have 5 or 6 fields separated by space.");
            };

            return Ok(Self {
                schedule: cron::Schedule::from_str(&expr)?,
                constraints: HashMap::new(),
            });
        }
        let caps = RE.captures(s).expect("invalid pattern");
        let shorthand = &caps["shorthand"];
        let cs = &caps["constraints"];

        let schedule = Schedule::from_str(shorthand)?;
        let mut constraints = HashMap::new();

        for constraint in cs.split(',') {
            let mut key_with_interval = constraint.splitn(2, '=');
            let key = key_with_interval.next().expect("timescale key");
            let interval: Interval = key_with_interval
                .next()
                .expect("interval")
                .parse()
                .expect("valid interval");

            tracing::trace!("Adding constraint at key {key}, value={interval:?}");
            constraints
                .entry(key.to_string())
                .or_insert(vec![])
                .push(interval);
        }

        Ok(Self {
            schedule,
            constraints,
        })
    }
}

impl CronWithRandomness {
    pub fn upcoming<'a, Z>(&'a self, timezone: Z) -> impl Iterator<Item = chrono::DateTime<Z>> + 'a
    where
        Z: chrono::TimeZone + 'a,
    {
        tracing::debug!("{:?}", self.constraints);
        self.schedule
            .upcoming(timezone)
            .map(|datetime| self.add_constraint(&datetime))
    }

    #[inline]
    fn add_constraint<Z>(&self, datetime: &chrono::DateTime<Z>) -> chrono::DateTime<Z>
    where
        Z: chrono::TimeZone,
    {
        let mut result_datetime = datetime.clone();

        // // pick a random minute. We have to reduce one hour from the hour range after this.
        // if noisy_minute {
        //     result_datetime += chrono::Duration::minutes(rng.gen_range(0..60));
        // }

        if let Some(hours) = self.constraints.get("h") {
            let chosen_internval = RNG.with_borrow_mut(|rng| hours.choose(rng).expect("chose one"));
            assert!((0..24).contains(&chosen_internval.0));
            assert!((0..24).contains(&chosen_internval.1));
            let dh = chosen_internval.random();
            tracing::debug!("Found h constraint: {dh:?}, chosen_internval: {chosen_internval:?}");
            result_datetime += chrono::Duration::hours(dh.into());
        }

        if let Some(days) = self.constraints.get("d") {
            let chosen_internval = RNG.with_borrow_mut(|rng| days.choose(rng).expect("chose one"));
            // 0 and 7 stands for Sunday in cron.
            assert!((0..8).contains(&chosen_internval.0));
            assert!((0..8).contains(&chosen_internval.1));
            let dh = chosen_internval.random();
            result_datetime += chrono::Duration::days(dh.into());
        }

        result_datetime
    }
}

/// Interval [low, high)
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

impl Interval {
    /// Generate a random value between the interval
    fn random(&self) -> i16 {
        // high is exclusive
        RNG.with_borrow_mut(|rng| rng.gen_range(self.0..self.1))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use chrono::Utc;
    use simple_accumulator::SimpleAccumulator;

    #[test]
    #[tracing_test::traced_test]
    fn test_cron_office() {
        use chrono::Timelike;

        let sch = CronWithRandomness::from_str("@daily{h=9-17,h=21-23}").unwrap();
        tracing::debug!("{sch:?}");

        let mut acc = SimpleAccumulator::with_fixed_capacity(&[], 10);

        let mut schedules = vec![];
        for datetime in sch.upcoming(Utc).take(100) {
            tracing::debug!("{datetime:?}");
            schedules.push(datetime);

            let time = datetime.time();
            assert!((9..17).contains(&time.hour()) || (21..23).contains(&time.hour()));
        }

        for i in 1..schedules.len() {
            let diff = schedules[i] - schedules[i - 1];
            let n_hours = diff.num_hours();
            tracing::debug!(" num hours = {n_hours}");
            assert!(diff.num_hours() < 48);
            assert!(diff.num_hours() > 1);
            acc.push(n_hours as f64);
        }

        println!(" {acc:?}");
        assert!(acc.mean() > 20.0);
        assert!(acc.mean() < 30.0);
        assert!(acc.variance() < 100.0);
        assert!(
            acc.variance() > 1.0,
            "Expecting some variance since values will be different. variance={}",
            acc.variance()
        );
    }

    #[test]
    fn test_cron_weekly() {
        use chrono::Datelike;
        use chrono::Timelike;

        let sch = CronWithRandomness::from_str("@weekly{d=1-3,h=21-23}").unwrap();
        println!("{sch:?}");

        let mut schedules = vec![];
        for datetime in sch.upcoming(Utc).take(100) {
            let weekday = datetime.weekday().num_days_from_sunday();
            let hour = datetime.time().hour();
            assert!([1, 2, 3].contains(&weekday));
            assert!([21, 22, 23].contains(&hour));
            schedules.push(datetime);
            println!("--> {datetime:?} weekday={weekday:?} hour={hour:?}");
        }
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_cron_sanity() {
        use chrono::Timelike;

        // this is standard cron expression.
        let sch = Schedule::from_str("@daily").unwrap();
        tracing::debug!("sch: {sch:#?}");

        let mut schedules = vec![];
        for datetime in sch.upcoming(Utc).take(10) {
            tracing::debug!("2-> {datetime:?}");

            schedules.push(datetime);
            assert_eq!(datetime.time().minute(), 0);
            assert_eq!(datetime.time().hour(), 0);
        }
        assert_eq!(schedules.len(), 10);
        for i in 1..schedules.len() {
            let diff = schedules[i] - schedules[i - 1];
            assert_eq!(diff.num_hours(), 24);
        }
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_cron_standard() {
        use chrono::Datelike;
        use chrono::Timelike;

        // second, min, hour, day, week, month
        let sch = CronWithRandomness::from_str("0 0/5 1/7 * *").unwrap();
        tracing::debug!("{sch:?}");
        for datetime in sch.upcoming(Utc).take(10) {
            tracing::debug!("2-> {datetime:?}");
            let time = datetime.time();
            assert_eq!(time.minute(), 0);
            assert_eq!(time.hour().rem_euclid(5), 0);

            let date = datetime.day();
            assert_eq!(date.rem_euclid(7), 1);
        }
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_cron_fixed_time() {
        use chrono::Timelike;

        // second, min, hour, day, week, month
        let sch = CronWithRandomness::from_str("0 12 * * *").unwrap();
        for datetime in sch.upcoming(Utc).take(10) {
            tracing::debug!("{datetime:?}, time: {:?}", datetime.time());
            assert_eq!(datetime.time().minute(), 0);
        }

        let sch = CronWithRandomness::from_str("* 12 * * *").unwrap();

        let mut min = 0;
        for datetime in sch.upcoming(Utc).take(10) {
            tracing::debug!("{datetime:?}, time: {:?}", datetime.time());
            assert_eq!(datetime.time().minute(), min);
            min += 1;
        }
    }
}
