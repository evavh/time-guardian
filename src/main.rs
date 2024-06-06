mod config;
mod counter;
mod file_io;
mod notification;
mod run;
mod user;
mod status;

const BREAK_IDLE_THRESHOLD: u64 = 10;

fn main() {
    run::run();
}
