use clap::{Args, Subcommand};

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
pub struct SettingsArgs {
    #[command(subcommand)]
    pub command: SettingsCommand,
}

/// Manage Codexi settings
#[derive(Subcommand, Debug)]
pub enum SettingsCommand {
    /// View codexi settings
    View,

    /// set codexi settings
    Set {
        /// Language
        #[arg(short = 'l', long, value_name = "LANGUAGE", help = "Language")]
        language: Option<String>,
        /// Currency
        #[arg(short = 'c', long, value_name = "CURRENCY", help = "Currency")]
        currency: Option<String>,
    },
}
