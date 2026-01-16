// src/core/wallet/system_kind.rs

use std::fmt;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Error type for SystemKind
#[derive(Debug, Error)]
pub enum SystemKindError {
    #[error("Unknown SystemKind type: '{0}'")]
    Unknown(String),
}
/// Enum representing the system kinds of operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum SystemKind {
    Init,
    Adjust,
    Close,
}
/// Methods for SystemKind
impl SystemKind {
    /// Get the string representation of the specific system kind
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemKind::Init => "Initialize",
            SystemKind::Adjust => "Adjust",
            SystemKind::Close => "Close",
        }
    }
    /// Try to create a SystemKind from a string
    pub fn try_from_str(s: &str) -> Result<Self, SystemKindError> {
        match s.to_ascii_lowercase().as_str() {
            "init" => Ok(SystemKind::Init),
            "adjust" => Ok(SystemKind::Adjust),
            "close" => Ok(SystemKind::Close),
            _ => Err(SystemKindError::Unknown(s.to_string())),
        }
    }
}
/// Implement TryFrom<&str> for SystemKind
impl TryFrom<&str> for SystemKind {
    type Error = SystemKindError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SystemKind::try_from_str(s)
    }
}
/// Implement From<SystemKind> for &'static str
impl From<SystemKind> for &'static str {
    fn from(t: SystemKind) -> Self {
        t.as_str()
    }
}
/// Implement Display for SystemKind
impl fmt::Display for SystemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<7}", <&'static str>::from(*self))
    }
}
