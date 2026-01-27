use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
#[clap(rename_all = "verbatim")]
pub enum CacheMode {
    Overwrite,
    NoCache,
}
