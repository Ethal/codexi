// src/command/account.rs

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
pub struct AccountArgs {
    #[command(subcommand)]
    pub command: AccountCommand,
}

/// Manage accounts (identity, lifecycle, and settings)
#[derive(Subcommand, Debug)]
pub enum AccountCommand {
    /// List all accounts (* = current, (c) = closed)
    List,

    /// Show the current account context
    Context,

    /// Create a new account
    Create {
        /// Opening date (YYYY-MM-DD)
        #[arg(value_name = "DATE", required = true, help = "Account opening date (YYYY-MM-DD)")]
        date: String,

        /// Account name (1+ words)
        #[arg(
            value_name = "NAME",
            required = true,
            num_args = 1..,
            help = "Account name (accepts multiple words)"
        )]
        name: Vec<String>,

        /// Account type
        #[arg(
            short = 't',
            long = "type",
            value_name = "ACCOUNT_TYPE",
            default_value = "Current",
            help = "Account type: Current, Joint, Saving, Deposit, Loan, Business, Student"
        )]
        account_type: Option<String>,
    },

    /// Switch active account
    Use {
        /// Account ID, short ID, or name
        #[arg(value_name = "ID", required = true, help = "Account ID, short ID, or name")]
        id: String,
    },

    /// [WARM] Close an account (irreversible)
    Close {
        /// Account ID, short ID, or name
        #[arg(
            index = 1,
            value_name = "ID",
            required = true,
            help = "Account ID, short ID, or name"
        )]
        id: String,

        /// Closing date (YYYY-MM-DD)
        #[arg(index = 2, value_name = "DATE", required = true, help = "Closing date (YYYY-MM-DD)")]
        date: String,
    },

    /// Rename an account
    Rename {
        /// Account ID, short ID, or name
        #[arg(
            index = 1,
            value_name = "ID",
            required = true,
            help = "Account ID, short ID, or name"
        )]
        id: String,

        /// New account name (1+ words)
        #[arg(
            index = 2,
            value_name = "NAME",
            required = true,
            help = "New account name (accepts multiple words)"
        )]
        name: Vec<String>,
    },

    /// Set bank for the current account
    SetBank {
        /// Bank ID, short ID, or name
        #[arg(value_name = "ID", required = true, help = "Bank ID, short ID, or name")]
        id: String,
    },

    /// Set currency for the current account
    SetCurrency {
        /// Currency ID, short ID, or ISO code
        #[arg(value_name = "ID", required = true, help = "Currency ID, short ID, or ISO code")]
        id: String,

        /// Update existing operations with this currency
        #[arg(short = 'u', long, help = "Update all existing operations")]
        update_operation: bool,
    },

    /// Set account context (limits, rules, constraints)
    SetContext {
        /// Overdraft limit (e.g., 500.00)
        #[arg(short = 'o', long, value_name = "AMOUNT", help = "Overdraft limit (e.g., 500.00)")]
        overdraft: Option<String>,

        /// Minimum balance (e.g., 100.00)
        #[arg(short = 'b', long, value_name = "AMOUNT", help = "Minimum balance (e.g., 100.00)")]
        balance_min: Option<String>,

        /// Max monthly transactions
        #[arg(short = 'm', long, value_name = "N", help = "Maximum number of monthly transactions")]
        max_monthly_transactions: Option<String>,

        /// Deposit lock date
        #[arg(short = 'd', long, value_name = "DATE", help = "Lock deposits until (YYYY-MM-DD)")]
        deposit_locked_until: Option<String>,

        /// Allow interest
        #[arg(short = 'i', long, help = "Enable interest")]
        interest: Option<bool>,

        /// Allow joint signers
        #[arg(short = 's', long, help = "Enable joint signers")]
        signers: Option<bool>,
    },
}
