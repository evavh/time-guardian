use color_eyre::Result;
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

#[cfg(feature = "deploy")]
pub(crate) mod path {
    pub(crate) const STATUS: &str = "/var/lib/time-guardian/status.ron";
    pub(crate) const CONFIG: &str = "/etc/time-guardian/config.ron";
    pub(crate) const PREV_CONFIG: &str = "/etc/time-guardian/prev-config.ron";
    pub(crate) const FALLBACK_CONFIG: &str =
        "/etc/time-guardian/fallback-config.ron";
    pub(crate) const TEMPLATE_CONFIG: &str =
        "/etc/time-guardian/template-config.ron";
    pub(crate) const RAMPEDUP: &str = "/var/lib/time-guardian/rampedup.ron";
}

#[cfg(not(feature = "deploy"))]
pub(crate) mod path {
    pub(crate) const STATUS: &str = "/var/lib/time-guardian-dev/status.ron";
    pub(crate) const CONFIG: &str = "/etc/time-guardian-dev/config.ron";
    pub(crate) const PREV_CONFIG: &str = "/etc/time-guardian-dev/prev-config.ron";
    pub(crate) const FALLBACK_CONFIG: &str =
        "/etc/time-guardian-dev/fallback-config.ron";
    pub(crate) const TEMPLATE_CONFIG: &str =
        "/etc/time-guardian-dev/template-config.ron";
    pub(crate) const RAMPEDUP: &str = "/var/lib/time-guardian-dev/rampedup.ron";
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
    Ok(ron::from_str(input).map_err(|e| Box::new(e))?)
}

pub(crate) fn to_string<T: Serialize>(object: &T) -> Result<String> {
    let extensions = Extensions::all();
    let config = PrettyConfig::new().extensions(extensions);
    Ok(ron::ser::to_string_pretty(object, config)?)
}
