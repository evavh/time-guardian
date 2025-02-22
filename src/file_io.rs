use color_eyre::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

#[cfg(feature = "deploy")]
pub(crate) mod path {
    pub(crate) const CONFIG: &str = "/etc/time-guardian/config.json";
    pub(crate) const PREV_CONFIG: &str = "/etc/time-guardian/prev-config.json";
    pub(crate) const FALLBACK_CONFIG: &str =
        "/etc/time-guardian/fallback-config.json";
    pub(crate) const TEMPLATE_CONFIG: &str =
        "/etc/time-guardian/template-config.json";

    pub(crate) const STATUS: &str = "/var/lib/time-guardian/status.json";
    pub(crate) const RAMPEDUP: &str = "/var/lib/time-guardian/rampedup.json";
}

// TODO: add windows paths
#[cfg(not(feature = "deploy"))]
pub(crate) mod path {
    pub(crate) const CONFIG: &str = "/etc/time-guardian-dev/config.json";
    pub(crate) const PREV_CONFIG: &str =
        "/etc/time-guardian-dev/prev-config.json";
    pub(crate) const FALLBACK_CONFIG: &str =
        "/etc/time-guardian-dev/fallback-config.json";
    pub(crate) const TEMPLATE_CONFIG: &str =
        "/etc/time-guardian-dev/template-config.json";

    pub(crate) const STATUS: &str = "/var/lib/time-guardian-dev/status.json";
    pub(crate) const RAMPEDUP: &str = "/var/lib/time-guardian-dev/rampedup.json";
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

pub(crate) fn load<T: DeserializeOwned>(path: &str) -> Result<T> {
    let data = std::fs::read_to_string(path)?;
    from_str(&data)
}

pub(crate) fn from_str<T: DeserializeOwned>(input: &str) -> Result<T> {
    Ok(serde_json::from_str(input)?)
}

pub(crate) fn to_string<T: Serialize>(object: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(object)?)
}
