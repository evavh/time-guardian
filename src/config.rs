use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{Duration, NaiveTime, Weekday};

const DEFAULT_CONFIG: &str = "<user>
   Monday 24h
      00:00-24:00 24h
   Tuesday 24h
      00:00-24:00 24h
   Wednesday 24h
      00:00-24:00 24h
   Thursday 24h
      00:00-24:00 24h
   Friday 24h
      00:00-24:00 24h
   Saturday 24h
      00:00-24:00 24h
   Sunday 24h
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

type WeekConfig = HashMap<Weekday, DayConfig>;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Config {
    users: HashMap<String, WeekConfig>,
}

peg::parser! {
    grammar config_parser() for str {
        rule username() -> String
            = s:$([^ '\n'|'\t']+) { s.to_owned() }

        rule mon() -> chrono::Weekday
            = s:$("monday") { Weekday::Mon }

        rule tue() -> chrono::Weekday
            = s:$("tuesday") { Weekday::Tue }

        rule wed() -> chrono::Weekday
            = s:$("wednesday") { Weekday::Wed }

        rule thu() -> chrono::Weekday
            = s:$("thursday") { Weekday::Thu }

        rule fri() -> chrono::Weekday
            = s:$("friday") { Weekday::Fri }

        rule sat() -> chrono::Weekday
            = s:$("saturday") { Weekday::Sat }

        rule sun() -> chrono::Weekday
            = s:$("sunday") { Weekday::Sun }

        rule weekday() -> chrono::Weekday
            = mon() / tue() / wed() / thu() / fri() / sat() / sun()

        rule number() -> u32
            = s:$(['0'..='9']*<1,2>) { s.parse().unwrap() }

        rule pos_number() -> i64
            = s:$(['0'..='9']*<1,2>) { s.parse().unwrap() }

        rule time_hm() -> NaiveTime
            = h:number() ":" m:number()
            { NaiveTime::from_hms_opt(h, m, 0).unwrap() }

        rule time_hms() -> NaiveTime
            = h:number() ":" m:number() ":" s:number()
            { NaiveTime::from_hms_opt(h, m, s).unwrap() }

        rule time() -> NaiveTime
            = time_hm() / time_hms()

        rule timerange() -> TimeRange
            = s:time() "-" e:time() { TimeRange { start: s, end: e}}

        rule duration_h() -> Duration
            = h:pos_number() "h" { Duration::hours(h) }

        rule duration_hm() -> Duration
            = h:pos_number() "h" m:pos_number() "m"
            { Duration::hours(h) + Duration::minutes(m) }

        rule duration_hms() -> Duration
            = h:pos_number() "h" m:pos_number() "m" s:pos_number() "s"
            { Duration::hours(h) + Duration::minutes(m) + Duration::seconds(s)}

        rule duration() -> Duration
            = duration_h() / duration_hm() / duration_hms()

        rule timeslot() -> Timeslot
            = t:timerange() " " d:duration()
            { Timeslot { range: t, duration: d }}

        rule timeslot_list() -> Vec<Timeslot>
            = timeslot() ++ "\n\t\t"

        rule dayconfig() -> DayConfig
            = d:duration() "\n\t\t" t:timeslot_list()
            { DayConfig { total_duration: d, timeslots: t }}

        rule weekconfig() -> WeekConfig
            = ("\t" w:weekday() " " c:dayconfig()) ** "\n"
            { todo!("Make vec of pairs and turn that into hashmap")}
    }
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
