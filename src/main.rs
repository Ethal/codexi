// src/main.rs

use std::env;
use anyhow::{Result};
use clap::{Parser};
use std::path::PathBuf;

mod core;

use crate::core::helpers::init_logger;
use crate::core::helpers::get_data_dir;
use crate::core::helpers::get_final_backup_path;
use crate::core::command::{
    Cli,
    Commands,
    ReportName,
    DataAction,
    SystemAction,
};
use crate::core::wallet::{
    Codexi,
    OperationKind,
    OperationFlow,
    RegularKind,
};

fn main() -> Result<()> {

    let cli = Cli::parse();

    let lvl = cli.verbose;
    init_logger(lvl);

    // current directory
    let cwd = env::current_dir()?;
    // app directory
    let data_dir = get_data_dir()?;

    let mut codexi = Codexi::load(&data_dir)?;

    match cli.command {

        Commands::Init { initial_amount, date } => {
            codexi.initialize(initial_amount, &date)?;
            codexi.save(&data_dir)?;
        },

        Commands::Debit { date, amount, description } => {
            codexi.add_operation(
                OperationKind::Regular(RegularKind::Transaction),
                OperationFlow::Debit,
                &date,
                amount,
                &description.join(" ")
            )?;
            codexi.save(&data_dir)?;
        },

        Commands::Credit { date, amount, description } => {
            codexi.add_operation(
                OperationKind::Regular(RegularKind::Transaction),
                OperationFlow::Credit,
                &date,
                amount,
                &description.join(" ")
            )?;
            codexi.save(&data_dir)?;
        },

        Commands::Rm { index } => {
            codexi.delete_operation(index)?;
            codexi.save(&data_dir)?;
        },

        Commands::Report(report_args) => {
            match report_args.report_name {
                ReportName::Balance { from, to, day, month, year } => {
                    let balance = codexi.balance(from, to, day, month, year)?;
                    Codexi::view_balance(&balance);
                },
                ReportName::Resume {} => {
                    let resume = codexi.resume()?;
                    Codexi::view_resume(&resume);
                },
            }
        },

        Commands::Search { from, to, text, kind, flow, day, amount_min, amount_max, latest } => {
            let results = codexi.search(
                from,
                to,
                text,
                kind,
                flow,
                day,
                amount_min,
                amount_max,
                latest,
            )?;

            Codexi::view_search(&results);
        },

        Commands::Data(data_args) => {
            match data_args.action {
                DataAction::Export(export_args) => {
                    if export_args.toml {
                        // export to readable format(toml)
                        codexi.export_toml(&cwd)?;
                    } else if export_args.csv {
                        // export to readable format(csv)
                        codexi.export_csv(&cwd)?;
                    }
                }
                DataAction::Import(import_args) => {
                    if import_args.toml {
                        let _ = codexi.snapshot();
                        // import from readable format(toml)
                        let codexi = Codexi::import_toml(&cwd)?;
                        codexi.save(&data_dir)?;
                    } else if import_args.csv {
                        let _ = codexi.snapshot();
                        // import from readable format(csv)
                        let codexi = Codexi::import_csv(&cwd)?;
                        codexi.save(&data_dir)?;
                    }
                }

                DataAction::RestoreSnapshot{ snapshot_file } => {
                    let codexi = Codexi::restore_snapshot(&snapshot_file)?;
                    codexi.save(&data_dir)?;
                }

                DataAction::ListSnapshot{} => {
                    let datas = Codexi::list_snapshot()?;
                    Codexi::view_snapshot(&datas);
                }

                DataAction::Snapshot{} => {
                    let _ = codexi.snapshot()?;
                }
            }
        },

        Commands::System(system_args) => {
            match system_args.action {
                SystemAction::Adjust { physical_balance, date} => {
                    codexi.adjust_balance(physical_balance, &date)?;
                    codexi.save(&data_dir)?;
                },
                SystemAction::Close { date, description } => {
                    codexi.close_period(&date, description)?;
                    codexi.save(&data_dir)?;
                },
                SystemAction::List {} => {
                    let results = Codexi::list_archives()?;
                    Codexi::view_archive(&results);
                },
                SystemAction::View {filename} => {
                    let codexi = Codexi::load_archive(&filename)?;
                    let results = codexi.search(None, None, None, None, None, None, None, None, None)?;
                    Codexi::view_search(&results);
                },
                SystemAction::Backup{ target_dir } => {
                    let final_backup_path = get_final_backup_path(target_dir.as_deref())?;
                    Codexi::backup(&final_backup_path)?;
                },
                SystemAction::Restore{ filename } => {
                    let full_path = PathBuf::from(filename);
                    Codexi::restore(&full_path)?;
                },
            }
        },
    }
    Ok(())
}
