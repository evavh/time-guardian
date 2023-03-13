use serde_derive::{Deserialize, Serialize};

const APP_NAME: &str = "time-guardian";
const CONFIG_NAME: &str = "config";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Config {
    pub(crate) allowed_times: Vec<(String, String)>,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            allowed_times: vec![("00:00".to_owned(), "24:00".to_owned())],
        }
    }
}

pub(crate) fn initialize() -> Config {
    // confy::store(APP_NAME, CONFIG_NAME, Config::default()).unwrap();
    let config: Config = confy::load(APP_NAME, CONFIG_NAME).unwrap();

    if config == Config::default() {
        let path =
            confy::get_configuration_file_path(APP_NAME, CONFIG_NAME).unwrap();
        println!("No user configuration found.");
        println!("The config file can be found at {}", path.display());
    }

    config
}
