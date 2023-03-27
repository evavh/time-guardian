use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{Duration, NaiveTime, Weekday};

const DEFAULT_CONFIG: &str = "<user>
   Monday
      00:00-24:00 24h
   Tuesday
      00:00-24:00 24h
   Wednesday
      00:00-24:00 24h
   Thursday
      00:00-24:00 24h
   Friday
      00:00-24:00 24h
   Saturday
      00:00-24:00 24h
   Sunday
      00:00-24:00 24h
";

#[derive(Debug, PartialEq, Eq, Clone)]
struct TimeRange {
    start: NaiveTime,
    end: NaiveTime,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Timeslot {
    range: TimeRange,
    duration: Duration,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct DayConfig {
    total_duration: Duration,
    timeslots: Vec<Timeslot>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Config {
    users: HashMap<String, HashMap<Weekday, DayConfig>>,
}

impl Config {
    fn from_string(file_contents: String) -> Result<Config, std::io::Error> {
        todo!()
    }

    fn load(path: &Path) -> Result<Config, std::io::Error> {
        let file_contents = String::from_utf8(fs::read(path)?).unwrap();
        Config::from_string(file_contents)
    }

    pub(crate) fn initialize(path: &Path) -> Config {
        match Config::load(path) {
            Ok(config) => config,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                fs::write(path, DEFAULT_CONFIG).unwrap();
                println!(
                    "No config found, writing default config to {}.",
                    path.display()
                );
                Config::load(path)
                    .expect("The config file should exist and be valid now.")
            }
            Err(e) => panic!("Unexpected IO error: {e}"),
        }
    }

    pub(crate) fn default() -> Config {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_config() {
        use chrono::Weekday::*;

        let parsed = Config::from_string(DEFAULT_CONFIG.to_owned());
        let total_duration = Duration::hours(24);

        let start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(24, 0, 0).unwrap();

        let range = TimeRange { start, end };
        let timeslot = Timeslot {
            range,
            duration: total_duration,
        };
        let timeslots = vec![timeslot];

        let dayconfig = DayConfig {
            total_duration,
            timeslots,
        };
        let days = HashMap::from([
            (Mon, dayconfig.clone()),
            (Tue, dayconfig.clone()),
            (Wed, dayconfig.clone()),
            (Thu, dayconfig.clone()),
            (Fri, dayconfig.clone()),
            (Sat, dayconfig.clone()),
            (Sun, dayconfig.clone()),
        ]);
        let users = HashMap::from([("<user>".to_owned(), days)]);
        let correct = Config { users };

        assert_eq!(parsed.unwrap(), correct);
    }
}
