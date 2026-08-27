// src/logic/account/policy/temporal.rs

use chrono::{Local, NaiveDate};
use nulid::Nulid;

use crate::logic::{
    account::Account,
    account::policy::TemporalViolation,
    operation::{OperationKind, SystemKind},
};

#[derive(Debug)]
pub enum TemporalAction {
    Create(OperationKind), // Create a new operation (Regular, Adjust, Checkpoint...)
    Void(Nulid),           // Void(op_id)
}

impl Account {
    /// Check that operations list is not empty — required before most operations.
    fn operations_empty(&self, kind: OperationKind) -> Result<(), TemporalViolation> {
        if self.operations.is_empty() {
            Err(TemporalViolation::HaveNoOperation(format!(
                "{}  count op: {} ",
                kind.as_str(),
                self.operations.len()
            )))
        } else {
            Ok(())
        }
    }

    /// Check that operations list is empty — required for Init.
    fn operations_not_empty(&self) -> Result<(), TemporalViolation> {
        if !self.operations.is_empty() {
            Err(TemporalViolation::HaveOperation)
        } else {
            Ok(())
        }
    }

    /// Returns an error if the account contains only an Init operation.
    /// Used by Checkpoint to ensure there is something to close.
    fn has_only_init(&self) -> Result<(), TemporalViolation> {
        let has_only_init = self.operations.len() == 1
            && self
                .operations
                .iter()
                .all(|op| matches!(op.kind, OperationKind::System(SystemKind::Init)));
        if has_only_init {
            return Err(TemporalViolation::OnlyInit);
        }
        Ok(())
    }

    /// Returns an error if the account is terminated (closed).
    /// No operation is allowed on a closed account.
    pub fn is_terminated(&self) -> Result<(), TemporalViolation> {
        if self.terminated_date.is_some() {
            return Err(TemporalViolation::AccountClose);
        }
        Ok(())
    }

    /// Validates the temporal policy for a given action.
    /// Called from action.rs before commit_operation().
    ///
    /// Two actions are supported:
    ///   - Create: validates date ordering, period locking, init sequencing
    ///   - Void:   validates that the target operation can be voided
    pub fn temporal_policy(&self, action: TemporalAction, date: NaiveDate) -> Result<(), TemporalViolation> {
        let today = Local::now().date_naive();

        // Account must be open for any operation
        self.is_terminated()?;

        // Extract anchors — no scan, uses cached values
        let last_init = self.anchors.last_init.as_ref();
        let last_chk = self.anchors.last_checkpoint.as_ref();
        let last_adj = self.anchors.last_adjust.as_ref();

        match action {
            TemporalAction::Create(kind) => {
                match kind {
                    // --- INITIALIZATION ---
                    // Only allowed on an empty account, not in the future.
                    OperationKind::System(SystemKind::Init) => {
                        self.operations_not_empty()?;
                        if date > today {
                            return Err(TemporalViolation::InvalidData(
                                "Init date cannot be in the future".into(),
                            ));
                        }
                    }

                    // --- CHECKPOINT ---
                    // Must be after last checkpoint, after init, not in the future,
                    // and there must be more than just an Init operation.
                    OperationKind::System(SystemKind::Checkpoint) => {
                        self.operations_empty(kind)?;
                        if date > today {
                            return Err(TemporalViolation::InvalidData(
                                "Closing date cannot be in the future".into(),
                            ));
                        }
                        if let Some(chk) = last_chk
                            && date <= chk.date
                        {
                            return Err(TemporalViolation::InvalidData(format!(
                                "Must be > last closing ({})",
                                chk.date
                            )));
                        }
                        if let Some(init) = last_init
                            && date < init.date
                        {
                            return Err(TemporalViolation::InvalidData("Close cannot be before Init".into()));
                        }
                        self.has_only_init()?;
                    }

                    // --- ADJUST ---
                    // Must not be in a closed period, must be >= last adjust date,
                    // not in the future.
                    OperationKind::System(SystemKind::Adjust) => {
                        self.operations_empty(kind)?;
                        if date > today {
                            return Err(TemporalViolation::InvalidData(
                                "Adjustment date cannot be in the future".into(),
                            ));
                        }
                        if let Some(chk) = last_chk
                            && date <= chk.date
                        {
                            return Err(TemporalViolation::InvalidData("Period closed".into()));
                        }
                        if let Some(adj) = last_adj
                            && date < adj.date
                        {
                            return Err(TemporalViolation::InvalidData(format!(
                                "Adjustment date must be >= {}",
                                adj.date
                            )));
                        }
                    }

                    // --- REGULAR OPERATION ---
                    // Must not be in a closed period.
                    // Must be >= last adjust or init date.
                    _ => {
                        self.operations_empty(kind)?;
                        if let Some(chk) = last_chk
                            && date <= chk.date
                        {
                            return Err(TemporalViolation::InvalidData("Period closed".into()));
                        }
                        let anchor_date = last_adj.map(|a| a.date).or_else(|| last_init.map(|a| a.date));
                        if let Some(a_dt) = anchor_date
                            && date < a_dt
                        {
                            return Err(TemporalViolation::InvalidData(format!("Must be >= {}", a_dt)));
                        }
                    }
                }
            }

            TemporalAction::Void(target_id) => {
                self.operations_empty(OperationKind::System(SystemKind::Void))?;

                // Void must be performed today or in the future — never backdated
                if date < today {
                    return Err(TemporalViolation::InvalidData(
                        "Void operation cannot be in the past".into(),
                    ));
                }

                let target_op = match self.get_operation_by_id(target_id) {
                    Some(op) => op,
                    None => {
                        return Err(TemporalViolation::OperationNotFound(target_id.to_string()));
                    }
                };

                // An operation cannot be voided twice
                if target_op.links.void_by.is_some() {
                    Err(TemporalViolation::OperationAlreadyVoided(target_id.to_string()))?;
                }

                // System operations (Init, Adjust, Checkpoint) cannot be voided
                if target_op.kind.is_system() {
                    return Err(TemporalViolation::InvalidData("System op cannot be voided".into()));
                }

                // --- Checkpoint lock ---
                // An operation is locked if its date is <= last checkpoint date.
                // Checkpoint is always strictly locking — no same-day exception.
                if let Some(chk) = last_chk
                    && target_op.date <= chk.date
                {
                    return Err(TemporalViolation::InvalidData(format!(
                        "Operation #{} is locked by checkpoint at {}.",
                        target_id, chk.date
                    )));
                }

                // --- Adjust lock ---
                // An operation is locked if it occurred before the last adjust.
                // Same-day operations are resolved by Nulid ordering:
                //   target_op.id < last_adjust.id → locked (op was before the adjust)
                //   target_op.id > last_adjust.id → allowed (op was after the adjust)
                if let Some(adj) = last_adj {
                    let locked = if target_op.date < adj.date {
                        // Strictly before adjust date — always locked
                        true
                    } else if target_op.date == adj.date {
                        // Same day — use Nulid ordering to determine if before or after
                        target_op.id < adj.id
                    } else {
                        // After adjust date — never locked
                        false
                    };

                    if locked {
                        return Err(TemporalViolation::InvalidData(format!(
                            "Operation #{} is locked by adjustment at {}.",
                            target_id, adj.date
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

/*------------------------ TEST ------------------------*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parse_date;
    use crate::logic::account::AccountType;
    use crate::logic::operation::{OperationFlow, RegularKind};
    use chrono::Duration;
    use rust_decimal_macros::dec;

    fn setup_empty_account() -> Account {
        Account::new(
            parse_date("2025-09-01").unwrap(),
            "Test".into(),
            AccountType::Current,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_void_locked_by_adjust() {
        let mut account = setup_empty_account();
        let today = Local::now().date_naive();
        let yesterday = today - Duration::days(1);

        // Init
        account.initialize(yesterday, dec!(100)).unwrap();

        // OP1 — before adjust
        let op_id = account
            .register_transaction(
                yesterday,
                OperationKind::Regular(RegularKind::Transaction),
                OperationFlow::Debit,
                dec!(25),
                "Grocery".into(),
                None,
                None,
            )
            .unwrap();

        // Adjust — locks OP1
        account.adjust_balance(today, dec!(50)).unwrap();

        // Void OP1 must fail — locked by adjust
        let res = account.temporal_policy(TemporalAction::Void(op_id), today);
        assert!(res.is_err(), "void should be locked by adjustment");
    }

    #[test]
    fn test_void_after_adjust_same_day_allowed() {
        let mut account = setup_empty_account();
        let today = Local::now().date_naive();

        // Init
        account.initialize(today, dec!(500)).unwrap();

        // OP1 — before adjust, same day
        account
            .register_transaction(
                today,
                OperationKind::Regular(RegularKind::Transaction),
                OperationFlow::Debit,
                dec!(50),
                "op1".into(),
                None,
                None,
            )
            .unwrap();

        // Adjust — same day
        account.adjust_balance(today, dec!(400)).unwrap();

        // OP3 — after adjust, same day
        let op3_id = account
            .register_transaction(
                today,
                OperationKind::Regular(RegularKind::Transaction),
                OperationFlow::Credit,
                dec!(100),
                "op3".into(),
                None,
                None,
            )
            .unwrap();

        // Void OP3 must succeed — it was inserted after the adjust
        let res = account.temporal_policy(TemporalAction::Void(op3_id), today);
        assert!(res.is_ok(), "void of op after adjust should be allowed");
    }

    #[test]
    fn test_void_before_adjust_same_day_locked() {
        let mut account = setup_empty_account();
        let today = Local::now().date_naive();

        // Init
        account.initialize(today, dec!(500)).unwrap();

        // OP1 — before adjust, same day
        let op1_id = account
            .register_transaction(
                today,
                OperationKind::Regular(RegularKind::Transaction),
                OperationFlow::Debit,
                dec!(50),
                "op1".into(),
                None,
                None,
            )
            .unwrap();

        // Adjust — same day, after OP1
        account.adjust_balance(today, dec!(400)).unwrap();

        // Void OP1 must fail — it was inserted before the adjust
        let res = account.temporal_policy(TemporalAction::Void(op1_id), today);
        assert!(res.is_err(), "void of op before adjust should be locked");
    }

    #[test]
    fn test_create_future_date_fail() {
        let account = setup_empty_account();
        let tomorrow = Local::now().date_naive() + Duration::days(1);

        let res = account.temporal_policy(
            TemporalAction::Create(OperationKind::System(SystemKind::Init)),
            tomorrow,
        );

        assert!(matches!(res, Err(TemporalViolation::InvalidData(msg)) if msg.contains("future")));
    }
}
