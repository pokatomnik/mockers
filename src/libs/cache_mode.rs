use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum CacheMode {
    Overwrite,
    NoCache,
}
