use clap::ValueEnum;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(ValueEnum, Clone, Debug, Serialize, PartialEq, PartialOrd)]
#[clap(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CacheMode {
    Overwrite,
    NoCache,
}

impl Display for CacheMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheMode::Overwrite => write!(f, "overwrite"),
            CacheMode::NoCache => write!(f, "nocache"),
        }
    }
}

impl FromStr for CacheMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "overwrite" => Ok(Self::Overwrite),
            "nocache" => Ok(Self::NoCache),
            _ => Err("Unknown cache mode"),
        }
    }
}

impl<'de> Deserialize<'de> for CacheMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
