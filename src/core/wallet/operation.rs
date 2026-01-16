// src/core/wallet/operation.rs

use std::fmt;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;
use thousands::Separable;

use super::operation_kind::OperationKind;
use super::operation_flow::OperationFlow;
use super::system_kind::SystemKind;
use super::regular_kind::RegularKind;

/// Error type for Operation
#[derive(Debug, Error)]
pub enum OperationError {
    #[error("Invalid Operation Date format: {0}")]
    InvalidDate(#[from] chrono::ParseError),
}
/// Struct representing a wallet operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub kind: OperationKind,
    pub flow: OperationFlow,
    pub date: NaiveDate,
    pub amount: f64,
    pub description: String,
}
/// Methods for Operation
impl Operation {

    pub fn new(
        kind: OperationKind,
        flow: OperationFlow,
        dt: &str,
        amount: f64,
        desc: impl Into<String>,
    ) -> Result<Self, OperationError>
    {

        let s: String = desc.into();
        let description = match s.trim() {
            "" => "no description".to_string(),
            t  => t.to_string(),
        };
        let naive_date = NaiveDate::parse_from_str(dt, "%Y-%m-%d")?;

        Ok(Self {
            kind: kind,
            flow: flow,
            date: naive_date,
            amount: amount,
            description:description,
        })
    }
    /// Create a new System Operation
    pub fn new_system_operation(
        kind: SystemKind,
        flow: OperationFlow,
        dt: &str,
        amount: f64,
        desc: impl Into<String>,
    ) -> Result<Self, OperationError>
    {
        Ok(Self::new(OperationKind::System(kind), flow, dt, amount,desc)?)

    }
    /// Create a new Regular Operation
    #[allow(dead_code)]
    pub fn new_regular_operation(
        kind: RegularKind,
        flow: OperationFlow,
        dt: &str,
        amount: f64,
        desc: impl Into<String>,
    ) -> Result<Self, OperationError>
    {
        Ok(Self::new(OperationKind::Regular(kind), flow, dt, amount,desc)?)
    }

}
/// Implement Display for Operation
impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {:.2} | {}",
            self.date.format("%Y-%m-%d"),
            self.kind,
            self.flow,
            self.amount.separate_with_commas(),
            self.description
        )
    }
}
