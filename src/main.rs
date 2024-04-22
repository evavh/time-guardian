use time_guardian::check_correct;

fn main() {
    // TODO: make correct path for root
    let config = confy::load("time-guardian", Some("config")).unwrap();
    check_correct(&config);

    time_guardian::run(&config)
}
