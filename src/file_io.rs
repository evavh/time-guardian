use color_eyre::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

pub(crate) mod path {
    pub(crate) const STATUS: &str = "/var/lib/time-guardian/status-dev.toml";
    pub(crate) const CONFIG: &str = "/etc/time-guardian/config-dev.toml";
    pub(crate) const PREV_CONFIG: &str =
        "/etc/time-guardian/prev-config-dev.toml";
    pub(crate) const FALLBACK_CONFIG: &str =
        "/etc/time-guardian/fallback-config-dev.toml";
    pub(crate) const TEMPLATE_CONFIG: &str =
        "/etc/time-guardian/template-config-dev.toml";
    pub(crate) const RAMPEDUP: &str =
        "/var/lib/time-guardian/rampedup-dev.toml";
}

pub(crate) fn store(
    object: &impl Serialize,
    path: &str,
) -> Result<(), std::io::Error> {
    let serialized = to_string(&object)
        .expect("Serializing failed, error in serializing format crate");

    if !PathBuf::from(path)
        .parent()
        .expect("This path should have a parent")
        .exists()
    {
        std::fs::create_dir_all(
            PathBuf::from(path)
                .parent()
                .expect("This path should have a parent"),
        )?;
    }
    std::fs::write(path, serialized)?;
    Ok(())
}

pub(crate) fn from_str<T: DeserializeOwned>(input: &str) -> Result<T> {
    Ok(toml::from_str(input).map_err(|e| Box::new(e))?)
}

pub(crate) fn to_string<T: Serialize>(object: &T) -> Result<String> {
    Ok(toml::to_string(object)?)
}
