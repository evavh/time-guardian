use time_guardian::{check_correct, Config, CONFIG_PATH};

fn main() {
    let config: Config = match confy::load_path(CONFIG_PATH) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Couldn't load config file at {CONFIG_PATH}, error: {e}");
            return;
        }
    };

    if let Err(e) = check_correct(&config) {
        eprintln!("Found error in config ({e}), aborting");
        return;
    }

    time_guardian::run(config)
}
