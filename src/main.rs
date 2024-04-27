use time_guardian::check_correct;

const CONFIG_PATH: &str = "/etc/time-guardian/config.toml";

fn main() {
    let config = confy::load_path(CONFIG_PATH).unwrap();
    check_correct(&config);

    time_guardian::run(&config)
}
