use color_eyre::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

pub(crate) mod path {
    use const_format::concatcp;

    #[cfg(feature = "deploy")]
    #[cfg(target_os = "linux")]
    const CONFIG_BASE: &str = "/etc/time-guardian/";
    #[cfg(feature = "deploy")]
    #[cfg(target_os = "linux")]
    const STATUS_BASE: &str = "/var/lib/time-guardian/";

    #[cfg(not(feature = "deploy"))]
    #[cfg(target_os = "linux")]
    const CONFIG_BASE: &str = "/etc/time-guardian-dev/";
    #[cfg(not(feature = "deploy"))]
    #[cfg(target_os = "linux")]
    const STATUS_BASE: &str = "/var/lib/time-guardian-dev/";

    #[cfg(feature = "deploy")]
    #[cfg(target_os = "windows")]
    const CONFIG_BASE: &str = "C:\\ProgramData\\time-guardian\\";
    #[cfg(feature = "deploy")]
    #[cfg(target_os = "windows")]
    const STATUS_BASE: &str = "C:\\ProgramData\\time-guardian\\";

    #[cfg(not(feature = "deploy"))]
    #[cfg(target_os = "windows")]
    const CONFIG_BASE: &str = "C:\\ProgramData\\time-guardian-dev\\";
    #[cfg(not(feature = "deploy"))]
    #[cfg(target_os = "windows")]
    const STATUS_BASE: &str = "C:\\ProgramData\\time-guardian-dev\\";

    const CONFIG_NAME: &str = "config.json";
    const PREV_CONFIG_NAME: &str = "prev-config.json";
    const FALLBACK_CONFIG_NAME: &str = "fallback-config.json";
    const TEMPLATE_CONFIG_NAME: &str = "template-config.json";
    const STATUS_NAME: &str = "status.json";
    const RAMPEDUP_NAME: &str = "rampedup.json";

    pub(crate) const CONFIG: &str = concatcp!(CONFIG_BASE, CONFIG_NAME);
    pub(crate) const PREV_CONFIG: &str =
        concatcp!(CONFIG_BASE, PREV_CONFIG_NAME);
    pub(crate) const FALLBACK_CONFIG: &str =
        concatcp!(CONFIG_BASE, FALLBACK_CONFIG_NAME);
    pub(crate) const TEMPLATE_CONFIG: &str =
        concatcp!(CONFIG_BASE, TEMPLATE_CONFIG_NAME);

    pub(crate) const STATUS: &str = concatcp!(STATUS_BASE, STATUS_NAME);
    pub(crate) const RAMPEDUP: &str = concatcp!(STATUS_BASE, RAMPEDUP_NAME);
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
