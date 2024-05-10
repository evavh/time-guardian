use color_eyre::{eyre::Context, Result};
use time_guardian::{check_correct, Config, CONFIG_PATH};

fn main() -> Result<()> {
    color_eyre::install()?;

    let config: Config = confy::load_path(CONFIG_PATH)
        .wrap_err("Couldn't load config file at {CONFIG_PATH}")?;

    check_correct(&config).wrap_err("Found error in config")?;

    time_guardian::run(config)
}
