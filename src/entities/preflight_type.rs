use clap::ValueEnum;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(ValueEnum, Copy, Clone, Debug, Serialize, PartialEq, PartialOrd)]
#[clap(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub(crate) enum PreflightType {
    /// Allow all headers/methods requested by browser
    Mirror,

    /// Allow ANYTHING: any headers, any methods — just make the browser shut up
    Permissive,
}

impl FromStr for PreflightType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "mirror" => Ok(Self::Mirror),
            "permissive" => Ok(Self::Permissive),
            _ => Err("Unknown preflight type"),
        }
    }
}

impl<'de> Deserialize<'de> for PreflightType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl Display for PreflightType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PreflightType::Mirror => write!(f, "mirror"),
            PreflightType::Permissive => write!(f, "permissive"),
        }
    }
}
