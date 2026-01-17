// src/ccore/wallet/codexi.rs

use anyhow::{Result, anyhow};
use std::fs;
use std::mem;

use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use chrono::{NaiveDate, Datelike};

use super::operation_flow::OperationFlow;
use super::operation_kind::OperationKind;
use super::system_kind::SystemKind;
use super::regular_kind::RegularKind;
use super::operation::Operation;
use crate::core::helpers::calculate_new_balance;
use crate::core::helpers::parse_flexible_date_range;
use crate::core::helpers::get_archive_path;
use crate::core::helpers::round_to_2_dec;

/// Struct for resume result
#[derive(Debug, Clone)]
pub struct ResumeResult {
    pub current_nb_transaction: usize,
    pub current_nb_init: usize,
    pub current_nb_adjust: usize,
    pub current_nb_close: usize,
    pub current_nb_op: usize,
    pub current_balance: f64,
    pub latest_transaction_date: String,
    pub latest_init_date: String,
    pub latest_adjust_date: String,
    pub latest_close_date: String,
}
/// Struct for balance result
#[derive(Debug, Clone)]
pub struct BalanceResult {
    pub credit: f64,
    pub debit: f64,
    pub total: f64,
}
/// Struct for search item
#[derive(Clone)]
pub struct SearchItem<'a> {
    pub index: i32,
    pub op: &'a Operation,
    pub balance: f64,
}
/// Struct representing the codexi
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Codexi {
    pub operations: Vec<Operation>,
}
/// Methods for codexi
impl Codexi {

    /// This function adds a new operation to the codexi while ensuring data integrity.
    /// ex: codexi.add_operation(...);
    /// It checks for date conflicts with existing system operations (Init, Close, Adjust)
    /// and ensures that debit operations do not exceed the current balance.
    pub fn add_operation(&mut self,
        kind:OperationKind,
        flow: OperationFlow,
        date: &str,
        amount: f64,
        description: &str,
    ) -> Result<()>
    {
        let new_op_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;

        let latest_close_date = self.operations.iter()
            .filter(|op| matches!(op.kind, OperationKind::System(SystemKind::Close)))
            .map(|op| op.date)
            .max();

        let latest_non_strict_date = self.operations.iter()
            .filter(|op| matches!(op.kind, OperationKind::System(SystemKind::Init) | OperationKind::System(SystemKind::Adjust)))
            .map(|op| op.date)
            .max();


        if let Some(close_date) = latest_close_date {
            if new_op_date <= close_date {
                log::error!(
                    "Operation date ({}) cannot be on or before the last period close date ({}).",
                    new_op_date, close_date
                );
                return Err(anyhow::anyhow!("Date conflict with period closure."));
            }
        }

        if let Some(anchor_date) = latest_non_strict_date {
            if new_op_date < anchor_date {
                log::error!(
                    "Operation date ({}) cannot be before the latest system anchor date ({}).",
                    new_op_date, anchor_date
                );
                return Err(anyhow::anyhow!("Date conflict with system anchor."));
            }
        }

        if flow == OperationFlow::Debit {
            let current_balance = self.balance(None, None, None, None, None)?.total;

            if current_balance < amount {
                log::error!("Debit operation cannot be added. Insufficient funds: Current balance is {} but debit amount is {}.",
                    current_balance,
                    amount
                );
                return Err(anyhow!("Date conflict with system anchor."));
            }
        }

        let op = Operation::new(kind, flow, date, amount, description)?;
        self.operations.push(op.clone());
        self.operations.sort_by_key(|o| o.date);
        log::info!("Operation added : {}", op);
        Ok(())
    }

    /// This function removes an operation at the specified index.
    /// ex: codexi.delete_operation(3);
    /// It checks if the operation is a system operation (Init, Close, Adjust) and prevents deletion if so.
    /// It returns an error if the index is out of bounds or if deletion is not allowed.
    pub fn delete_operation(&mut self, index: usize) -> Result<()> {

        if index >= self.operations.len() {
            return Err(anyhow::anyhow!("Operation index {} is out of bounds.", index));
        }

        let op_kind = self.operations[index].kind;

        if matches!(
            op_kind,
            OperationKind::System(SystemKind::Init) |
            OperationKind::System(SystemKind::Close) |
            OperationKind::System(SystemKind::Adjust))
        {
            return Err(anyhow::anyhow!(
                "Operation #{} cannot be deleted: it is a protected system entry (Initial Balance, Adjustment or Carried Forward Solde).",
                index
            ));
        }

        self.operations.remove(index);
        log::info!("Operation #{} successfully removed.", index);

        Ok(())
    }

    /// Sets the initial balance of the codexi.
    /// ex: codexi.initialize(1000.0, "2024-07-01");
    /// This function creates an initial operation representing the starting balance.
    /// It should only be called when the codexi is empty.
    pub fn initialize(
        &mut self,
        amount: f64,
        date_str: &str,
    ) -> Result<()>
    {
        if !self.operations.is_empty() {
            return Err(anyhow::anyhow!("The codexi is not empty. Cannot set initial balance."));
        }

        let op_flow = OperationFlow::from_sign(amount);
        let description = format!("INITIAL AMOUNT");

        // 3. Créer l'opération
        self.add_operation(
            OperationKind::System(SystemKind::Init) ,
            op_flow,
            &date_str,
            amount.abs(), // Utiliser la valeur absolue
            &description,
        )?;

        log::info!("codexi initialized with a balance of {} on {}.", amount, date_str);
        Ok(())
    }

    /// This function adjusts the codexi to match a physical balance.
    /// It calculates the difference and creates an adjustment operation if needed.
    /// Negative physical balances are not allowed.
    /// ex: codexi.adjust_balance(950.0, "2024-07-15");
    pub fn adjust_balance(
        &mut self,
        physical_balance: f64,
        date_str: &str,
    ) -> Result<()>
    {

        if physical_balance < 0.0 {
            log::warn!("Negative physical balance not allow.");
            return Ok(());
        }

        let current_balance = self.balance(None, None, None, None, None)?.total;

        let difference = physical_balance - current_balance;

        if difference.abs() < 0.001 {
            log::info!("No adjustment needed. Theoretical balance ({}) matches physical balance ({}).",
                    current_balance, physical_balance);
            return Ok(());
        }

        let adjustment_flow = OperationFlow::from_sign(difference);
        let adjustment_amount = difference.abs();

        let description = format!("ADJUSTMENT: Deviation of {} to reach physical balance {}",
                                adjustment_amount, physical_balance);

        self.add_operation(
            OperationKind::System(SystemKind::Adjust),
            adjustment_flow,
            &date_str,
            adjustment_amount,
            &description,
        )?;

        log::warn!("ADJUSTMENT MADE: Added a {} of {} to correct the balance.",
                adjustment_flow,
                adjustment_amount,
        );

        Ok(())
    }

    /// This function closes the current accounting period by archiving all operations
    /// up to the specified closing date and creating a new "Carried Forward Solde" operation.
    /// ex: codexi.close_period("2024-07-31", vec!["End of July".to_string()]);
    /// It saves the archived operations to a file and updates the codexi accordingly.
    /// The description_parts are concatenated to describe the closing operation.
    pub fn close_period(
        &mut self,
        close_date_str: &str,
        description_parts: Vec<String>,
    ) -> Result<()>
    {
        let close_date = NaiveDate::parse_from_str(close_date_str, "%Y-%m-%d")?;

        let mut current_closing_balance: f64 = 0.0;
        let mut archived_operations = Vec::new();

        let original_operations = mem::take(&mut self.operations);

        for op in original_operations.into_iter() {
            let op_date = op.date;

            if op_date <= close_date {

                match op.kind {
                    OperationKind::System(SystemKind::Init) | OperationKind::System(SystemKind::Close) => {
                        archived_operations.push(op.clone());

                        match op.flow {
                            OperationFlow::Credit => current_closing_balance = op.amount,
                            OperationFlow::Debit => current_closing_balance = -op.amount,
                            OperationFlow::None => {},
                        }
                    }
                    OperationKind::System(SystemKind::Adjust) |
                    OperationKind::Regular(RegularKind::Transaction) |
                    OperationKind::Regular(RegularKind::Fee) |
                    OperationKind::Regular(RegularKind::Transfer) |
                    OperationKind::Regular(RegularKind::Refund) => {
                        match op.flow {
                            OperationFlow::Credit => current_closing_balance += op.amount,
                            OperationFlow::Debit => current_closing_balance -= op.amount,
                            OperationFlow::None => {},
                        }
                        archived_operations.push(op);
                    }
                }
            } else {
                self.operations.push(op);
            }
        }

        // If there's nothing to close, we stop.
        if archived_operations.is_empty() && self.operations.iter().all(|op| !matches!(op.kind,
            OperationKind::System(SystemKind::Init) |
            OperationKind::System(SystemKind::Close)))
        {
            // Management logic if the codexi is empty or contains only previous anchors.
            // If there are no transactions to archive, nothing is done.
            log::info!("No transactions (Adjust/Others) found to archive on or before {}.", close_date_str);
            return Ok(());
        }

        // --- PART 1: ARCHIVE MANAGEMENT ---

        // Save the archive if there are transactions to archive.
        if !archived_operations.is_empty() {
            let archive_path = get_archive_path(close_date_str)?;
            let encoded_archive = bincode::serialize(&archived_operations)?;
            fs::write(&archive_path, encoded_archive)?;
            log::info!("Archived {} operations to {:?}", archived_operations.len(), archive_path);
        }

        // --- PART 2: CREATION OF THE NEW ANCHOR ---

        let net_solde = current_closing_balance;

        // 1. Create the new Carry Forward Balance operation
        let new_flow = OperationFlow::from_sign(net_solde);
        let new_amount = net_solde.abs();
        let description = format!("SOLDE REPORTÉ : {} {}", new_amount, description_parts.join(" "));

        let new_op = Operation::new_system_operation(
            SystemKind::Close,
            new_flow,
            close_date_str,
            new_amount,
            description,
        )?;

        // 2. Add the new anchor to the vector.
        // This new anchor replaces all old anchors and transactions up to close_date.
        self.operations.push(new_op);

        // 3. Sort the final vector (so that the new anchor is in the correct position)
        // We sort by both date and type to resolve conflicts on the same day.
        self.operations.sort_by(|a, b| {
            // Primary sorting by date
            let date_order = a.date.cmp(&b.date);
            if date_order != Ordering::Equal {
                return date_order;
            }
            // Secondary sorting for equal dates
            a.kind.cmp(&b.kind)
        });

        log::warn!("PERIOD CLOSED: All transactions up to {} archived and replaced by single Close entry.", close_date_str);

        Ok(())
    }

    /// Get the operations with balance
    pub fn get_operations_with_balance(&self) -> Vec<(&Operation, f64)> {
        let mut cur_bal = 0.0;
        let mut out = Vec::new();

        for op in &self.operations {
            cur_bal = calculate_new_balance(cur_bal, op).unwrap_or(0.0);
            out.push((op, cur_bal));
        }

        out
    }

    /// Calculates the total of credits, debits and the final balance,
    /// with several date filters (from/to/day/month/year).
    /// Returns a BalanceResult struct.
    pub fn balance(
        &self,
        from: Option<String>,
        to: Option<String>,
        day: Option<String>,
        month: Option<String>,
        year: Option<String>,
    ) -> Result<BalanceResult> {

        // Cumulated value
        let mut credit: f64 = 0.0;
        let mut debit: f64 = 0.0;
        let mut total: f64 = 0.0;

        // Parsing from/to
        let start_date = from
            .as_deref()
            .map(|d| parse_flexible_date_range(d, true))
            .transpose()?;

        let end_date = to
            .as_deref()
            .map(|d| parse_flexible_date_range(d, false))
            .transpose()?;

        // Expected format : "YYYY-MM-DD"
        let filter_day: Option<NaiveDate> = match day.as_deref() {
            Some(dstr) => match NaiveDate::parse_from_str(dstr, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => return Ok(BalanceResult{credit: 0.0, debit: 0.9, total: 0.0}), // jour invalide = aucun match
            },
            None => None,
        };

        // Expected format : "YYYY-MM"
        let filter_month: Option<(i32, u32)> = if let Some(m) = month.as_deref() {
            let parts: Vec<&str> = m.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(y), Ok(mo)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                    Some((y, mo))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Expected format : "YYYY"
        let filter_year: Option<i32> = match year.as_deref() {
            Some(ystr) => match ystr.parse::<i32>() {
                Ok(v) => Some(v),
                Err(_) => return Ok(BalanceResult{credit: 0.0, debit: 0.9, total: 0.0}), // année invalide = aucun match
            },
            None => None,
        };

        for op in self.operations.iter() {

            // --- Filter FROM
            if let Some(s_date) = start_date {
                if op.date < s_date {
                    continue;
                }
            }

            // --- Filter TO
            if let Some(e_date) = end_date {
                if op.date > e_date {
                    continue;
                }
            }

            // --- Filter EXACT DAY
            if let Some(d) = filter_day {
                if op.date != d {
                    continue;
                }
            }

            // --- Filter MONTH
            if let Some((y, m)) = filter_month {
                if op.date.year() != y || op.date.month() != m {
                    continue;
                }
            }

            // --- Filter YEAR
            if let Some(y) = filter_year {
                if op.date.year() != y {
                    continue;
                }
            }

            // --- Cumulate CREDIT / DEBIT
            match op.flow {
                OperationFlow::Credit => credit += op.amount,
                OperationFlow::Debit  => debit  += op.amount,
                OperationFlow::None   => {},
            }

            total = credit - debit;
        }

        credit = round_to_2_dec(credit);
        debit = round_to_2_dec(debit);
        total = round_to_2_dec(total);

        Ok(BalanceResult{ credit, debit, total })
    }

    /// Search
    /// Returns a vector of SearchItem
    pub fn search(
        &self,
        from: Option<String>,
        to: Option<String>,
        text: Option<String>,
        kind: Option<String>,
        flow: Option<String>,
        day: Option<String>,
        amount_min: Option<f64>,
        amount_max: Option<f64>,
        latest: Option<usize>,
    ) -> Result<Vec<SearchItem<'_>>> {

        let ops_map = self.get_operations_with_balance();

        let start_date = from
            .as_deref()
            .map(|d| parse_flexible_date_range(d, true))
            .transpose()?;

        let end_date = to
            .as_deref()
            .map(|d| parse_flexible_date_range(d, false))
            .transpose()?;

        let text_lc = text.as_ref().map(|t| t.to_lowercase());

        let o_flow_filter = match flow {
            Some(ref s) => match OperationFlow::try_from(s.as_str()) {
                Ok(v) => Some(v),
                Err(_) => return Ok(Vec::new()),
            },
            None => None,
        };

        let o_kind_filter = match kind {
            Some(ref s) => match OperationKind::try_from(s.as_str()) {
                Ok(v) => Some(v),
                Err(_) => return Ok(Vec::new()),
            },
            None => None,
        };

        let day_parsed = match day.as_deref() {
            Some(dstr) => match NaiveDate::parse_from_str(dstr, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => return Ok(Vec::new()),
            },
            None => None,
        };

        let mut matched: Vec<SearchItem> = Vec::new();

        for (idx, &(op, bal)) in ops_map.iter().enumerate() {
            // from
            if let Some(s_date) = start_date {
                if op.date < s_date {
                    continue;
                }
            }

            // to
            if let Some(e_date) = end_date {
                if op.date > e_date {
                    continue;
                }
            }

            if let Some(ref needle) = text_lc {
                if !op.description.to_lowercase().contains(needle) {
                    continue;
                }
            }

            if let Some(f_op) = o_flow_filter {
                if op.flow != f_op {
                    continue;
                }
            }

            if let Some(k_op) = o_kind_filter {
                if op.kind != k_op {
                    continue;
                }
            }

            if let Some(d) = day_parsed {
                if op.date != d {
                    continue;
                }
            }

            if let Some(min) = amount_min {
                if op.amount < min {
                    continue;
                }
            }

            if let Some(max) = amount_max {
                if op.amount > max {
                    continue;
                }
            }

            matched.push(SearchItem {
                index: idx as i32,
                op,
                balance: bal,
            });
        }

        let result = if let Some(n) = latest {
            if matched.len() <= n {
                matched
            } else {
                let start = matched.len().saturating_sub(n);
                matched[start..].to_vec()
            }
        } else {
            matched
        };

        Ok(result)
    }
    /// Resume
    /// Returns a ResumeResult struct
    pub fn resume(&self) -> Result<ResumeResult> {
        let mut nb_transaction: usize = 0;
        let mut nb_init: usize = 0;
        let mut nb_adjust: usize = 0;
        let mut nb_close: usize = 0;
        let mut latest_transaction_date = String::from("__________");
        let mut latest_init_date = String::from("__________");
        let mut latest_adjust_date = String::from("__________");
        let mut latest_close_date = String::from("__________");

        for op in &self.operations {
            match op.kind {
                OperationKind::Regular(RegularKind::Transaction) => {
                    nb_transaction += 1;
                    latest_transaction_date = op.date.format("%Y-%m-%d").to_string();
                }
                OperationKind::System(SystemKind::Init) => {
                    nb_init += 1;
                    latest_init_date = op.date.format("%Y-%m-%d").to_string();
                }
                OperationKind::System(SystemKind::Adjust) => {
                    nb_adjust += 1;
                    latest_adjust_date = op.date.format("%Y-%m-%d").to_string();
                }
                OperationKind::System(SystemKind::Close) => {
                    nb_close += 1;
                    latest_close_date = op.date.format("%Y-%m-%d").to_string();
                }
                _ => { /* Ignore other types of operations */ }
            }
        }
        let current_balance = self.balance(None, None, None, None, None)?.total;
        let nb_op = nb_transaction + nb_init + nb_adjust + nb_close;

        Ok(ResumeResult {
            current_nb_transaction: nb_transaction,
            current_nb_init: nb_init,
            current_nb_adjust: nb_adjust,
            current_nb_close: nb_close,
            current_nb_op: nb_op,
            current_balance,
            latest_transaction_date,
            latest_init_date,
            latest_adjust_date,
            latest_close_date,
        })
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    fn setup_empty_codexi() -> Codexi {
        // init
        Codexi::default()
    }

    // Helper function to initialize with known data
    fn setup_codexi_with_data() -> Codexi {
        let mut cb = Codexi::default();

        // #4 Credit (2025-11-05) : 100.00
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Credit,
            "2025-11-05".to_string().as_str(),
            100.0,
            format!("Atm").as_str(),
        ).unwrap();

        // #1 Credit (2025-10-08) : 50.00
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Credit,
            "2025-10-08".to_string().as_str(),
            50.0,
            format!("Atm").as_str(),
        ).unwrap();

        // #7 Debit (2025-12-05) : 25.50
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Debit,
            "2025-12-05".to_string().as_str(),
            25.50,
            format!("Minimarket").as_str(),
        ).unwrap();

        // #0 Debit (2025-10-04) : 14.20
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Debit,
            "2025-10-04".to_string().as_str(),
            14.20,
            format!("Book").as_str(),
        ).unwrap();

        // #2 Debit (2025-10-21) : 44.80
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Debit,
            "2025-10-21".to_string().as_str(),
            44.80,
            format!("Post office").as_str(),
        ).unwrap();

        // #9 Credit (2025-12-15) : 150.00
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Credit,
            "2025-12-15".to_string().as_str(),
            150.0,
            format!("Atm").as_str(),
        ).unwrap();

        // #5 Debit (2025-11-12) : 15.70
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Debit,
            "2025-11-12".to_string().as_str(),
            15.70,
            format!("Bakery").as_str(),
        ).unwrap();

        // #3 Debit (2025-10-21) : 11.00
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Debit,
            "2025-10-21".to_string().as_str(),
            11.00,
            format!("Fruits").as_str(),
        ).unwrap();

        // #8 Credit (2025-12-10) : 10.00
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Credit,
            "2025-12-10".to_string().as_str(),
            10.0,
            format!("Refund").as_str(),
        ).unwrap();

        // #6 Debit (2025-11-20) : 23.60
        cb.add_operation(
            OperationKind::Regular(RegularKind::Transaction),
            OperationFlow::Debit,
            "2025-11-20".to_string().as_str(),
            23.60,
            format!("Newspapers").as_str(),
        ).unwrap();

        cb
    }

    #[test]
    fn test_default_codexi_is_empty() -> Result<()> {
        let codexi = setup_empty_codexi();

        assert_eq!(codexi.operations.len(), 0, "The default codexi should have 0 operations.");

        let balance_result = codexi.balance(None, None, None, None, None)?;
        assert_eq!(balance_result.total, 0.0, "The balance of an empty codexi must be 0.0.");

        Ok(())
    }


    #[test]
    fn test_full_account_balance() -> Result<()> {
        let codexi = setup_codexi_with_data();

        let balance_result = codexi.balance(None, None, None, None, None)?;

        // ASSERT: Verification of expected results
        // Expected total balance: 310.00 - 134.80 = 175.20
        // Expected total credit: 100.00 + 50.00 + 150.00 + 10.00 = 310.00
        // Expected total debit: 25.50 + 14.20 + 44.80 + 15.70 + 11.00 + 23.60 = 134.80

        assert_eq!(balance_result.credit, 310.00, "The total credits are incorrect");
        assert_eq!(balance_result.debit, 134.80, "The total debits are incorrect.");
        assert_eq!(balance_result.total, 175.20, "The final account balance is incorrect.");

        Ok(())
    }


    #[test]
    fn test_balance_with_range_filter() -> Result<()> {
        let codexi = setup_codexi_with_data();

        let balance_result = codexi.balance(
            Some("2025-12-04".to_string()), // --from (start_date)
            Some("2025-12-06".to_string()), // --to (end_date)
            None, None, None
        )?;

        assert_eq!(balance_result.credit, 0.00, "The total filtered credit must be 0.0.");
        assert_eq!(balance_result.debit, 25.50, "The total debits are incorrect.");
        assert_eq!(balance_result.total, -25.50, "The balance filtered by date range is incorrect.");

        Ok(())
    }

    #[test]
    fn test_balance_with_day_filter_no_operations() -> Result<()> {
        let codexi = setup_codexi_with_data();

        let balance_result = codexi.balance(
            None,
            None,
            Some("2025-12-06".to_string()), // --day
            None,
            None,
        )?;

        assert_eq!(balance_result.credit, 0.00, "The total filtered credit must be 0.0.");
        assert_eq!(balance_result.debit, 0.00, "The total filtered debit must be 0.0.");
        assert_eq!(balance_result.total, 0.00, "The balance filtered by date range is incorrect.");

        Ok(())
    }

    #[test]
    fn test_balance_with_filter_month() -> Result<()> {
        let codexi = setup_codexi_with_data();

        let balance_result = codexi.balance(
            None,
            None,
            None,
            Some("2025-11".to_string()), // --month
            None,
        )?;

        assert_eq!(balance_result.credit, 100.00, "The total credits are incorrect.");
        assert_eq!(balance_result.debit, 39.30, "The total debits are incorrect");
        assert_eq!(balance_result.total, 60.70, "The balance filtered by date range is incorrect.");

        Ok(())
    }
}
