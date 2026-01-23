use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub enum MockersRouteError {
    IncorrectHTTPMethod,
}

impl Error for MockersRouteError {}

impl fmt::Display for MockersRouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockersRouteError::IncorrectHTTPMethod => write!(f, "IncorrectHTTPMethod"),
        }
    }
}
