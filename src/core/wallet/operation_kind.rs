// src/core/wallet/operation_kind.rs

use std::fmt;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use super::system_kind::SystemKind;
use super::regular_kind::RegularKind;

/// Error type for OperationKind
#[derive(Debug, Error)]
pub enum OperationKindError {
    #[error("Unknown OperationKind type: '{0}'")]
    Unknown(String),
}
/// Enum representing the kind of operation: System or Regular
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum OperationKind {
    System(SystemKind),
    Regular(RegularKind),
}
/// Methods for OperationKind
impl OperationKind {
    /// Check if the OperationKind is a System kind
    pub fn is_system(&self) -> bool {
        matches!(self, OperationKind::System(_))
    }
    /// Check if the OperationKind is a Regular kind
    pub fn is_regular(&self) -> bool {
        matches!(self, OperationKind::Regular(_))
    }
    /// Get the type of OperationKind as a string
    pub fn kind_type(&self) -> &'static str {
        match self {
            OperationKind::System(_) => "System",
            OperationKind::Regular(_) => "Regular",
        }
    }
    /// Get the string representation of the specific kind
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationKind::System(kind) => kind.as_str(),
            OperationKind::Regular(kind) => kind.as_str(),
        }
    }
    /// Try to create an OperationKind from a string
    pub fn try_from_str(s: &str) -> Result<Self, OperationKindError> {
        let lower = s.to_ascii_lowercase();

        if let Ok(sk) = SystemKind::try_from_str(&lower) {
            return Ok(OperationKind::System(sk));
        }

        if let Ok(rk) = RegularKind::try_from_str(&lower) {
            return Ok(OperationKind::Regular(rk));
        }

        Err(OperationKindError::Unknown(lower.to_string()))
    }   
}
/// Implement TryFrom<&str> for OperationKind
impl TryFrom<&str> for OperationKind {
    type Error = OperationKindError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        OperationKind::try_from_str(s)
    }
}

/// Implement From<OperationKind> for &'static str
impl From<OperationKind> for &'static str {
    fn from(t: OperationKind) -> Self {
        t.as_str()
    }
}
/// Implement Display for OperationKind
impl fmt::Display for OperationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationKind::System(system_kind) => write!(f, "System::{system_kind}"),
            OperationKind::Regular(regular_kind) => write!(f, "Regular::{regular_kind}"),
        }
    }
}
