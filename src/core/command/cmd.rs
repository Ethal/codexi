// scr/core/command/cmd.rs
use clap::{Parser, ArgGroup, Args, Subcommand };
use chrono::Local;

#[derive(Parser, Debug)]
#[command(author="ethal", version="1.O.0")]
pub struct Cli {
    /// Verbose
    #[arg(short, long, global = true, help = "Increase verbosity level")]
    pub verbose: bool,
    /// Command
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {

    /// Initializes the codexi with a starting balance.
    Init {
        /// The initial account balance.
        #[arg(index = 1, value_name = "INITIAL_BALANCE", required = true, allow_negative_numbers = false)]
        initial_amount: f64,

        /// The start date of the initialization (YYYY-MM-DD).
        #[arg(index = 2, value_name = "DATE", default_value_t = Local::now().date_naive().to_string())]
        date: String,
    },

    /// Add a regular debit operation
    Debit {
        #[arg(index = 1, value_name = "DATE", required = true, help = "Date of the debit operation (YYYY-MM-DD)")]
        date: String,

        #[arg(index = 2, value_name = "AMOUNT", required = true, help = "Amount of the debit operation", allow_negative_numbers = false )]
        amount: f64,

        #[arg(index = 3, value_name = "DESCRIPTION...", help = "Description of the debit operation", default_value = "no description")]
        description: Vec<String>,
    },

    /// Add a regular credit operation
    Credit {
        #[arg(index = 1, value_name = "DATE", required = true, help = "Date of the credit operation (YYYY-MM-DD)")]
        date: String,

        #[arg(index = 2, value_name = "AMOUNT", required = true, help = "Amount of the credit operation", allow_negative_numbers = false)]
        amount: f64,

        #[arg(index = 3, value_name = "DESCRIPTION...", help = "Description of the credit operation", default_value = "no description")]
        description: Vec<String>,
    },

    /// Remove an operation by index.
    Rm {
        #[arg(value_name = "INDEX", help = "Index of the operation to remove", allow_negative_numbers = false)]
        index: usize
    },

    /// Search in operation.
    Search {
        // Filtres granulaire (Plage de dates arbitraire)
        #[arg(long, help = "Start date for filtering operations", value_name = "FROM_DATE")]
        from: Option<String>,

        #[arg(long, help = "End date for filtering operations", value_name = "TO_DATE")]
        to: Option<String>,

        /// Filter by text contained in description
        #[arg(short = 't', long, help = "Filter by text in description", value_name = "TEXT")]
        text: Option<String>,

        /// Filter by type of kind operation (Init, Adjust, Close, Transaction, ...)
        #[arg(short = 'k', long, help = "Filter by kind: 'init', 'adjust', 'close', 'transaction', 'fee', 'transfer', 'refund'", value_name = "KIND")]
        kind: Option<String>,

        /// Filter by the flow of operation (debit, credit)
        #[arg(short = 'f', long, help = "Filter by flow: 'debit' or 'credit'", value_name = "FLOW")]
        flow: Option<String>,

        /// Filter by a specific day (YYYY-MM-DD)
        #[arg(short = 'd', long, value_name = "YYYY-MM-DD", help = "Filter by specific day (YYYY-MM-DD)")]
        day: Option<String>,

        /// Minimum amount
        #[arg(long = "a-min", help = "Minimum amount", value_name = "AMOUNT", allow_negative_numbers = false)]
        amount_min: Option<f64>,

        /// Maximum amount
        #[arg(long = "a-max", help = "Maximum amount", value_name = "AMOUNT", allow_negative_numbers = false)]
        amount_max: Option<f64>,

        /// The latest operations to display.
        #[arg(long, help = "The latest N operations to display", value_name = "NUMBER", allow_negative_numbers = false)]
        latest: Option<usize>,
    },

    /// Report.
    Report(ReportArgs),

    /// Export/Import/Snapshot/Backup.
    Data(DataArgs),

    /// Manages accounting anchors (Initial Balance, Adjustment, Closing).
    System(SystemArgs),

}

#[derive(Parser, Debug)]
pub struct ReportArgs {
    #[command(subcommand)]
    pub report_name: ReportName, // ReportName contient Balance, etc.
}

#[derive(Subcommand, Debug)]
pub enum ReportName {
    /// Show the balance and debit/credit. Available criteria, --from --to --day, --month, --year.
    Balance {
        // Filtres granulaire (Plage de dates arbitraire)
        #[arg(long, value_name = "YYYY-MM-DD, YYYY-MM, YYYY", help = "Start date for filtering operations", value_name = "FROM_DATE")]
        from: Option<String>,

        #[arg(long, value_name = "YYYY-MM-DD, YYYY-MM, YYYY", help = "End date for filtering operations", value_name = "TO_DATE")]
        to: Option<String>,

        // Optionnel : balance pour une journée spécifique (Ex: -d 2025-11-24)
        #[arg(short = 'd', long, value_name = "YYYY-MM-DD", help = "Filter by specific day (YYYY-MM-DD)")]
        day: Option<String>,

        // Optionnel : balance pour un mois spécifique (Ex: -m 2025-11)
        #[arg(short = 'm', long, value_name = "YYYY-MM", help = "Filter by specific month (YYYY-MM)")]
        month: Option<String>,

        // Optionnel : balance pour une année spécifique (Ex: -y 2025)
        #[arg(short = 'y', long, value_name = "YYYY", help = "Filter by specific year (YYYY)")]
        year: Option<String>,
    },
    /// Show the codexi resume.
    Resume {},
}

// Nouvelle structure DataArgs
#[derive(Parser, Debug)]
pub struct DataArgs {
    #[command(subcommand)]
    pub action: DataAction,
}

#[derive(Subcommand, Debug)]
pub enum DataAction {
    /// Export the data to an external format (CSV, TOML)
    #[command(group = ArgGroup::new("format").required(true))]
    Export(ExportArgs),

    /// Importing data from an external format (CSV, TOML)
    #[command(group = ArgGroup::new("format").required(true))]
    Import(ImportArgs),

    /// Performed a snapshot
    Snapshot {},

    /// list the available snapshot
    ListSnapshot {},

    /// Restore a snapshot
    RestoreSnapshot {
        #[arg(value_name = "SNAPSHOT_FILE", help = "Used 'ListSnapShot' for the available snapshot files")]
        snapshot_file: String,
    },
}

#[derive(Args, Debug)]
pub struct ExportArgs {

    /// Export to csv format
    #[arg(short = 'c', long, conflicts_with = "toml", group = "format", help = "Export to CSV format")]
    pub csv: bool,

    /// Export to toml format
    #[arg(short = 't', long, conflicts_with = "csv", group = "format", help = "Export to TOML format")]
    pub toml: bool,
}

#[derive(Args, Debug)]
pub struct ImportArgs {

    /// Import from csv format
    #[arg(short = 'c', long, conflicts_with = "toml", group = "format", help = "Import from CSV format")]
    pub csv: bool,

    /// Import from toml format
    #[arg(short = 't', long, conflicts_with = "csv", group = "format", help = "Import from TOML format")]
    pub toml: bool,
}

// structure System
#[derive(Parser, Debug)]
pub struct SystemArgs {
    #[command(subcommand)]
    pub action: SystemAction,
}

#[derive(Subcommand, Debug)]
pub enum SystemAction {
    /// Adjusts the codexi balance to a given physical amount.
    Adjust {
        /// The actual physical balance.
        #[arg(index = 1, value_name = "PHYSICAL_BALANCE", allow_negative_numbers = false, help = "The actual physical balance to adjust the codexi to this amount.")]
        physical_balance: f64,

        /// The start date of the initialization (YYYY-MM-DD).
        #[arg(index = 2, value_name = "DATE", default_value_t = Local::now().date_naive().to_string(), help = "The date of the adjustment (YYYY-MM-DD).")]
        date: String,
    },

    /// Closes operations up to the specified date, replacing them with a carried-over balance.
    Close {
        /// The closing date (YYYY-MM-DD). All transactions prior to this date will be archived and deleted from the codexi.
        #[arg(value_name = "DATE", required = true, help = "The closing date (YYYY-MM-DD). All transactions prior to this date will be archived and deleted from the codexi.")]
        date: String,

        /// Description of the balance carried forward (ex: 'Closing Year 2025').
        #[arg(value_name = "DESCRIPTION...", help = "Description of the closing operation")]
        description: Vec<String>,
    },

    /// List the archive file
    List {},

    /// View the content of an archive file
    View {
        /// Load an archieve file (view only)
        #[arg(value_name = "FILENAME", help = "The archive filename to view")]
        filename: String,
    },

    /// Backup datas
    Backup {
        #[arg(long, value_name = "DIR or PATH", help = "Target directory or full path for the backup ZIP file. If a directory is provided, a default filename with timestamp will be used.")]
        target_dir: Option<String>,
    },

    /// Restore datas from a backup file
    Restore {
        #[arg(value_name = "FILENAME", help = "The backup ZIP filename to restore from")]
        filename: String,
    },

}
