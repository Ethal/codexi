// src/handler/data.rs

use anyhow::Result;
use std::path::Path;

use codexi::{
    core::DataPaths,
    exchange::Exchangeable,
    file_management::{ExchangeSerdeFormat, FileExchangeError, FileManagement},
    logic::{
        account::Account, category::CategoryList, counterparty::CounterpartyList, currency::CurrencyList,
        operation::AccountOperations,
    },
};

use crate::{
    command::{DataCommand, ExchangeFormat, ExchangeTypeCommand, SnapshotCommand},
    msg_info, msg_warn,
    prompts::Prompt,
    ui::{view_snapshot, view_warning},
};

pub fn handle_data_command(command: DataCommand, cwd: &Path, paths: &DataPaths, skip_confirm: bool) -> Result<()> {
    let mut codexi = FileManagement::load_current_state(paths)?;
    match command {
        DataCommand::Export(exchange_type) => match exchange_type.command {
            ExchangeTypeCommand::AccountHeader { format } => {
                let account = codexi.get_current_account_mut()?;
                export_with_format(account, format, cwd)?;
                msg_info!("Export accounts header completed");
            }
            ExchangeTypeCommand::Operation { format } => {
                let account = codexi.get_current_account()?;
                export_with_format(&account.to_account_operations(), format, cwd)?;
                msg_info!("Export operations completed");
            }
            ExchangeTypeCommand::Currency { format } => {
                export_with_format(&codexi.currencies, format, cwd)?;
                msg_info!("Export currencies completed");
            }
            ExchangeTypeCommand::Category { format } => {
                export_with_format(&codexi.categories, format, cwd)?;
                msg_info!("Export categories completed");
            }
            ExchangeTypeCommand::Counterparty { format } => {
                export_with_format(&codexi.counterparties, format, cwd)?;
                msg_info!("Export counterparties completed");
            }
        },
        DataCommand::Import(exchange_type) => match exchange_type.command {
            ExchangeTypeCommand::AccountHeader { format } => {
                if !skip_confirm && !Prompt::confirm("Import the data?", false)? {
                    msg_info!("Command cancelled.");
                    return Ok(());
                }
                let (account, warnings) = import_with_format::<Account>(format, cwd)?;
                let summary = codexi.import_account_header(account)?;
                FileManagement::save_current_state(&codexi, paths)?;
                msg_info!(
                    "Import in {}: {} created, {} updated.",
                    summary.name,
                    summary.created,
                    summary.updated
                );
                if !warnings.is_empty() {
                    view_warning(&warnings);
                    msg_warn!("Import accounts header completed, {} warnings", warnings.len());
                } else {
                    msg_info!("Import accounts header completed");
                }
                msg_warn!("It is recommended to run `admin audit --rebuild` to verify data integrity.");
            }
            ExchangeTypeCommand::Operation { format } => {
                if !skip_confirm && !Prompt::confirm("Import the data?", false)? {
                    msg_info!("Command cancelled.");
                    return Ok(());
                }
                let (account_operations, mut warnings) = import_with_format::<AccountOperations>(format, cwd)?;
                let (summary, merge_warnings) = codexi.import_operations(account_operations)?;
                warnings.extend(merge_warnings);
                FileManagement::save_current_state(&codexi, paths)?;
                msg_info!(
                    "Import in {}: {} created, {} updated.",
                    summary.name,
                    summary.created,
                    summary.updated
                );
                if !warnings.is_empty() {
                    view_warning(&warnings);
                    msg_warn!("Import operations completed, {} warnings", warnings.len());
                } else {
                    msg_info!("Import operations completed");
                }
                msg_warn!("It is recommended to run `admin audit --rebuild` to verify data integrity.");
            }
            ExchangeTypeCommand::Currency { format } => {
                if !skip_confirm && !Prompt::confirm("Import the data?", false)? {
                    msg_info!("Command cancelled.");
                    return Ok(());
                }
                let (currencies, warnings) = import_with_format::<CurrencyList>(format, cwd)?;
                let summary = codexi.import_currencies(currencies)?;
                FileManagement::save_current_state(&codexi, paths)?;
                msg_info!(
                    "Import in {}: {} created, {} updated.",
                    summary.name,
                    summary.created,
                    summary.updated
                );
                if !warnings.is_empty() {
                    view_warning(&warnings);
                    msg_warn!("Import currencies completed, {} warnings", warnings.len());
                } else {
                    msg_info!("Import currencies completed");
                }
            }
            ExchangeTypeCommand::Category { format } => {
                if !skip_confirm && !Prompt::confirm("Import the data?", false)? {
                    msg_info!("Command cancelled.");
                    return Ok(());
                }
                let (categories, warnings) = import_with_format::<CategoryList>(format, cwd)?;
                let summary = codexi.import_categories(categories)?;
                FileManagement::save_current_state(&codexi, paths)?;
                msg_info!(
                    "Import in {}: {} created, {} updated.",
                    summary.name,
                    summary.created,
                    summary.updated
                );
                if !warnings.is_empty() {
                    view_warning(&warnings);
                    msg_warn!("Import categories completed, {} warnings", warnings.len());
                } else {
                    msg_info!("Import categories completed");
                }
            }
            ExchangeTypeCommand::Counterparty { format } => {
                if !skip_confirm && !Prompt::confirm("Import the data?", false)? {
                    msg_info!("Command cancelled.");
                    return Ok(());
                }
                let (counterparties, warnings) = import_with_format::<CounterpartyList>(format, cwd)?;
                let summary = codexi.import_counterparties(counterparties)?;
                FileManagement::save_current_state(&codexi, paths)?;
                msg_info!(
                    "Import in {}: {} created, {} updated.",
                    summary.name,
                    summary.created,
                    summary.updated
                );
                if !warnings.is_empty() {
                    view_warning(&warnings);
                    msg_warn!("Import counterparties completed, {} warnings", warnings.len());
                } else {
                    msg_info!("Import counterparties completed");
                }
            }
        },
        DataCommand::Snapshot(snapshot) => match snapshot.command {
            SnapshotCommand::Create => {
                FileManagement::create_snapshot(&codexi, paths)?;
                msg_info!("Snapshot done");
            }
            SnapshotCommand::List => {
                let snapshot = FileManagement::list_snapshot(paths)?;
                view_snapshot(&snapshot);
            }
            SnapshotCommand::Restore { snapshot_file } => {
                if !skip_confirm && !Prompt::confirm("Restore the snapshot ?", false)? {
                    msg_info!("Command cancelled.");
                    return Ok(());
                }
                let codexi = FileManagement::restore_snapshot(paths, &snapshot_file)?;
                FileManagement::save_current_state(&codexi, paths)?;
                msg_info!("Restore of the snapshot ({}) completed", snapshot_file);
            }

            SnapshotCommand::Clean { keep } => {
                FileManagement::clean_snapshot(paths, keep)?;
                msg_info!("Cleaning of snapshot done");
            }
        },
    }
    Ok(())
}

// export/import helper
fn export_with_format<T: Exchangeable>(data: &T, format: ExchangeFormat, cwd: &Path) -> Result<(), FileExchangeError> {
    match format {
        ExchangeFormat::Json => ExchangeSerdeFormat::Json.export(data, cwd),
        ExchangeFormat::Toml => ExchangeSerdeFormat::Toml.export(data, cwd),
        ExchangeFormat::Csv => Err(FileExchangeError::UnsupportedFormat),
    }
}

fn import_with_format<T: Exchangeable>(
    format: ExchangeFormat,
    cwd: &Path,
) -> Result<(T, Vec<T::Warning>), FileExchangeError> {
    match format {
        ExchangeFormat::Json => ExchangeSerdeFormat::Json.import(cwd),
        ExchangeFormat::Toml => ExchangeSerdeFormat::Toml.import(cwd),
        ExchangeFormat::Csv => Err(FileExchangeError::UnsupportedFormat),
    }
}
