//! Binary to generate patterns.

use chrono::Utc;
use cron_with_randomness::CronWithRandomness;
use std::str::FromStr;

fn main() {
    let pattern = std::env::args().nth(1).expect("no pattern given");
    let num_samples = std::env::args()
        .nth(2)
        .unwrap_or("10".to_string())
        .parse::<usize>()
        .unwrap();

    let schedule = CronWithRandomness::from_str(&pattern).unwrap();

    for datetime in schedule.upcoming(Utc).take(num_samples) {
        println!(" --> {datetime:?}");
    }
}
