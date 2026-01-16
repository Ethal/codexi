// src/core/wallet/operation_flow.rs

use std::fmt;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Error type for OperationFlow
#[derive(Debug, Error)]
pub enum OperationFlowError {
    #[error("Unknown OperationFlow type: '{0}'")]
    Unknown(String),
}
/// Enum representing the flow of an operation: Debit or Credit
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OperationFlow {
    Debit,
    Credit,
    None,
}
/// Methods for OperationFlow
impl OperationFlow {
    /// Get the string representation of the specific flow
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationFlow::Debit => "Debit",
            OperationFlow::Credit => "Credit",
            OperationFlow::None => "None",
        }
    }
    /// Try to create an OperationFlow from a string
    pub fn try_from_str(s: &str) -> Result<Self, OperationFlowError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "debit" | "db"  => Ok(OperationFlow::Debit),
            "credit" | "cr" => Ok(OperationFlow::Credit),
            "none" | "no"   => Ok(OperationFlow::None),
            _ => Err(OperationFlowError::Unknown(s.to_string())),
        }
    }
    /// Check if the OperationFlow is Debit or Credit
    pub fn is_debit(&self) -> bool {
        matches!(self, OperationFlow::Debit)
    }
    /// Check if the OperationFlow is Credit
    pub fn is_credit(&self) -> bool {
        matches!(self, OperationFlow::Credit)
    }
    /// Check if the OperationFlow is None
    pub fn is_none(&self) -> bool {
        matches!(self, OperationFlow::None)
    }
    /// Get the opposite flow
    pub fn opposite(&self) -> Self {
        match self {
            OperationFlow::Debit => OperationFlow::Credit,
            OperationFlow::Credit => OperationFlow::Debit,
            OperationFlow::None => OperationFlow::None,
        }
    }
    /// Toggle the flow in place
    pub fn toggle(&mut self) {
        *self = self.opposite();
    }
    /// Get the sign associated with the flow
    pub fn to_sign(&self) -> f64 {
        match self {
            OperationFlow::Debit => -1.0,
            OperationFlow::Credit => 1.0,
            OperationFlow::None => 0.0,
        }
    }
    /// Create an OperationFlow from a sign
    pub fn from_sign(sign: f64) -> Self {
        if sign > 0.0 {
            OperationFlow::Credit
        } else if sign < 0.0 {
            OperationFlow::Debit
        } else {
            OperationFlow::None
        }
    }
}
/// Implement TryFrom<&str> for OperationFlow
impl TryFrom<&str> for OperationFlow {
    type Error = OperationFlowError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        OperationFlow::try_from_str(value)
    }
}
/// Implement From<OperationFlow> for &'static str
impl From<OperationFlow> for &'static str {
    fn from(t: OperationFlow) -> Self {
        t.as_str()
    }
}
/// Implement Display for OperationFlow
impl fmt::Display for OperationFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<7}", self.as_str())
    }
}
