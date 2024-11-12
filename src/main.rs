//! Binary to generate patterns.

use chrono::Utc;
use cron_with_randomness::CronWithRandomness;
use std::str::FromStr;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let pattern = std::env::args().nth(1).expect("no pattern given");
    let num_samples = std::env::args()
        .nth(2)
        .unwrap_or("10".to_string())
        .parse::<usize>()
        .unwrap();

    let schedule = CronWithRandomness::from_str(&pattern).unwrap();
    tracing::debug!("Schedule: {schedule:?}");

    for datetime in schedule.upcoming(Utc).take(num_samples) {
        println!(" --> {datetime:?}");
    }
}
