pub mod configuration_api;
pub mod console_log;
pub mod deps;
pub mod fork;
pub mod formatter;
pub mod node;
pub mod resolver;
pub mod utils;
pub mod zks;
use std::{fmt::Display, str::FromStr};

use clap::Parser;
pub use zksync_types::l2::L2Tx;

#[derive(Debug, Parser, Clone, clap::ValueEnum, PartialEq, Eq)]
pub enum ShowCalls {
    None,
    User,
    System,
    All,
}

impl FromStr for ShowCalls {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "none" => Ok(ShowCalls::None),
            "user" => Ok(ShowCalls::User),
            "system" => Ok(ShowCalls::System),
            "all" => Ok(ShowCalls::All),
            _ => Err(()),
        }
    }
}

impl Display for ShowCalls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
