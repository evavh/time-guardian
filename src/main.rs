use time_guardian::check_correct;


fn main() {
    let config = confy::load_path(time_guardian::CONFIG_PATH).unwrap();
    check_correct(&config).unwrap();

    time_guardian::run(config)
}
