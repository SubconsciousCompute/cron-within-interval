//! cron-office
//! Extended cron expressions for Office and WFH folks.
//!
//! In addition to standard expression supported by excellent crate cron, we support following type
//! of expressions.
//!
//! @daily{9-17} means run once between 9am and 5pm (at any time).  @daily{9-12,15-20} means runs
//! once either between 9am and 12pm or between 3pm and 8pm.
//!
//! Similarly one can pass daily contraints to @weekly
//!
//! @weekly{1-5} runs once per week between day 1 and day 5.
//!
//! @weekly{1-5/9-12} runs once per week between day 1 and day 5 and sometimes between 9am and
//! 12pm.

/// Wrapper around cron::Schedule
pub struct CronOffice {
    inner: cron::Schedule,
}

impl std::std::FromStr for CronOffice {
    type Err = anyhow::Err;

    fn parse(s: &str) -> anyhow::Result<Self> {
        Ok(Self {
            inner: cron::Schedule::parse(s)?,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_sanity() {
    }
}
