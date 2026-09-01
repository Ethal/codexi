// src/command/root.rs

use clap::{Parser, Subcommand};

use crate::command::account::AccountArgs;
use crate::command::admin::AdminArgs;
use crate::command::bank::BankArgs;
use crate::command::category::CategoryArgs;
use crate::command::counterparty::CounterpartyArgs;
use crate::command::currency::CurrencyArgs;
use crate::command::data::DataArgs;
use crate::command::history::HistoryArgs;
use crate::command::loan::LoanArgs;
use crate::command::operation::OperationArgs;
use crate::command::report::ReportArgs;
use crate::command::settings::SettingsArgs;

#[derive(Parser, Debug)]
#[command(
    name = "codexi-cli",
    version,
    about = "Codexi - Personal financial ledger & analytics",
    long_about = r#"
Codexi is an append-only financial ledger designed for clarity,
traceability, and long-term analysis.

Typical workflow:
  - Record operations (debit/credit/transfer/interest)
  - Search and filter data
  - Generate reports (dashboard, financial, monthly, etc.)
  - Manage snapshots, archives, and backups
"#,
    arg_required_else_help = true
)]
pub struct Cli {
    #[arg(long, help = "Enable TUI mode")]
    pub tui: bool,

    #[arg(short, long, global = true, help = "Skip confirmation prompts")]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Option<RootCommand>,
}

#[derive(Subcommand, Debug)]
pub enum RootCommand {
    /// List all accounts with their balance, debit, and credit
    Overview,

    /// Switch to another account (alias for `account use`)
    Use {
        /// Account ID, short ID, or name
        #[arg(
            value_name = "ID",
            required = true,
            help = "Account ID. Accepts full ID, short ID, or name"
        )]
        id: String,
    },

    /// Add a debit operation
    Debit {
        #[arg(index = 1, value_name = "DATE", required = true, help = "Date (YYYY-MM-DD)")]
        date: String,

        #[arg(
            index = 2,
            value_name = "AMOUNT",
            required = true,
            allow_negative_numbers = false,
            help = "Amount (must be positive)"
        )]
        amount: String,

        #[arg(
            index = 3,
            value_name = "DESCRIPTION",
            required = false,
            num_args = 0..,
            help = "Operation description"
        )]
        description: Vec<String>,

        #[arg(
            short = 'c',
            long = "counterparty",
            value_name = "COUNTERPARTY",
            help = "Counterparty (ID, short ID, or name)"
        )]
        counterparty: Option<String>,

        #[arg(
            short = 'g',
            long = "category",
            value_name = "CATEGORY",
            help = "Category (ID, short ID, or name)"
        )]
        category: Option<String>,
    },

    /// Add a credit operation
    Credit {
        #[arg(index = 1, value_name = "DATE", required = true, help = "Date (YYYY-MM-DD)")]
        date: String,

        #[arg(
            index = 2,
            value_name = "AMOUNT",
            required = true,
            allow_negative_numbers = false,
            help = "Amount (must be positive)"
        )]
        amount: String,

        #[arg(
            index = 3,
            value_name = "DESCRIPTION",
            required = false,
            num_args = 0..,
            help = "Operation description"
        )]
        description: Vec<String>,

        #[arg(
            short = 'c',
            long = "counterparty",
            value_name = "COUNTERPARTY",
            help = "Counterparty (ID, short ID, or name)"
        )]
        counterparty: Option<String>,

        #[arg(
            short = 'g',
            long = "category",
            value_name = "CATEGORY",
            help = "Category (ID, short ID, or name)"
        )]
        category: Option<String>,
    },

    /// Add an interest operation
    Interest {
        #[arg(index = 1, value_name = "DATE", required = true, help = "Date (YYYY-MM-DD)")]
        date: String,

        #[arg(
            index = 2,
            value_name = "AMOUNT",
            required = true,
            allow_negative_numbers = false,
            help = "Amount (must be positive)"
        )]
        amount: String,

        #[arg(
            index = 3,
            value_name = "DESCRIPTION",
            required = false,
            num_args = 0..,
            help = "Operation description"
        )]
        description: Vec<String>,

        #[arg(
            short = 'c',
            long = "counterparty",
            value_name = "COUNTERPARTY",
            help = "Counterparty (ID, short ID, or name)"
        )]
        counterparty: Option<String>,

        #[arg(
            short = 'g',
            long = "category",
            value_name = "CATEGORY",
            help = "Category (ID, short ID, or name)"
        )]
        category: Option<String>,
    },

    /// Transfer funds to another account. Supports full ID, short ID, or name
    Transfer {
        #[arg(index = 1, value_name = "DATE", required = true, help = "Date (YYYY-MM-DD)")]
        date: String,

        #[arg(
            index = 2,
            value_name = "AMOUNT_FROM",
            required = true,
            allow_negative_numbers = false,
            help = "Amount sent from current account (positive)"
        )]
        amount_from: String,

        #[arg(
            index = 3,
            value_name = "AMOUNT_TO",
            required = true,
            allow_negative_numbers = false,
            help = "Amount received by destination account (positive)"
        )]
        amount_to: String,

        #[arg(
            index = 4,
            value_name = "ACCOUNT_ID_TO",
            required = true,
            help = "Destination account (ID, short ID, or name)"
        )]
        account_id_to: String,

        #[arg(
            index = 5,
            value_name = "DESCRIPTION",
            num_args = 0..,
            help = "Operation description"
        )]
        description: Vec<String>,

        #[arg(
            short = 'g',
            long = "category",
            value_name = "CATEGORY",
            help = "Category (ID, short ID, or name)"
        )]
        category: Option<String>,
    },

    /// Search and filter operations (alias: `view`)
    #[command(alias = "view")]
    Search {
        #[arg(long, value_name = "FROM_DATE", help = "Start date (YYYY-MM-DD)")]
        from: Option<String>,

        #[arg(long, value_name = "TO_DATE", help = "End date (YYYY-MM-DD)")]
        to: Option<String>,

        #[arg(short = 't', long, value_name = "TEXT", help = "Filter by description text")]
        text: Option<String>,

        #[arg(
            short = 'k',
            long,
            value_name = "KIND",
            help = "Filter by kind: 'init', 'adjust', 'void', 'checkpoint', 'transaction', 'fee', 'transfer', 'interest', 'refund'"
        )]
        kind: Option<String>,

        #[arg(short = 'f', long, value_name = "FLOW", help = "Filter by flow: 'debit' or 'credit'")]
        flow: Option<String>,

        #[arg(
            short = 'c',
            long,
            value_name = "COUNTERPARTY",
            help = "Filter by counterparty (ID, short ID, or name)"
        )]
        counterparty: Option<String>,

        #[arg(
            short = 'g',
            long,
            value_name = "CATEGORY",
            help = "Filter by category (ID, short ID, or name)"
        )]
        category: Option<String>,

        #[arg(
            long = "a-min",
            value_name = "AMOUNT",
            allow_negative_numbers = true,
            help = "Minimum amount"
        )]
        amount_min: Option<String>,

        #[arg(
            long = "a-max",
            value_name = "AMOUNT",
            allow_negative_numbers = true,
            help = "Maximum amount"
        )]
        amount_max: Option<String>,

        #[arg(
            long,
            value_name = "NUMBER",
            allow_negative_numbers = false,
            help = "Show the latest N operations"
        )]
        last: Option<usize>,

        #[arg(long, help = "Show today's operations")]
        today: bool,

        #[arg(long, help = "Open result in the default browser")]
        open: bool,
    },

    /// Codexi Settings
    Settings(SettingsArgs),

    /// Manage operations
    Operation(OperationArgs),

    /// Manage accounts
    Account(AccountArgs),

    /// Manage banks
    Bank(BankArgs),

    /// Manage currencies
    Currency(CurrencyArgs),

    /// Manage counterparties
    Counterparty(CounterpartyArgs),

    /// Manage categories
    Category(CategoryArgs),

    /// Manage accounting timeline (init, checkpoint, adjust, void, close)
    History(HistoryArgs),

    /// Generate financial reports (dashboard, balance, monthly, etc.)
    Report(ReportArgs),

    /// Import/export data and manage snapshots
    Data(DataArgs),

    /// [Warn] Technical maintenance, disaster recovery, and low-level file management.
    /// Use with caution; run `--help` for details.
    Admin(AdminArgs),

    /// Simulate loan interest and repayment (linear or compound)
    Loan(LoanArgs),
}
