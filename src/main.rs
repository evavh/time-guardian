use clap::{Parser, Subcommand};

mod config;
mod counter;
mod file_io;
mod notification;
mod run;
mod status;
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
    #[cfg(feature = "deploy")]
    println!("Deploying");

    let cli = Cli::parse();

    match cli.command {
        Command::Run => run::run(),
        Command::Spent { user } => status::spent(&user),
        Command::Status { user } => status::status(&user),
    }
}
