use clap::{Parser, Subcommand};

mod config;
mod file_io;
mod logging;
mod notification;
mod run;
#[cfg(target_os = "windows")]
mod session;
mod status;
mod time_slot;
mod tracker;
mod user;

const BREAK_IDLE_THRESHOLD: u64 = 10;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Monitor and enforce time
    Run,
    /// Print machine-readable spent time in seconds
    Spent { user: String },
    /// Print human-readable time left message
    Status { user: String },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Run => run::run(),
        Command::Spent { user } => status::spent(&user),
        Command::Status { user } => status::status(&user),
    }
}
