// src/handler/history.rs

use anyhow::Result;

use codexi::{
    core::{DataPaths, parse_date, parse_decimal, parse_id, parse_text},
    dto::SearchOperationCollection,
    file_management::FileManagement,
    logic::{
        account::AccountError,
        operation::Operation,
        search::{SearchParamsBuilder, search},
        utils::resolve_id,
    },
};

use crate::{
    command::{ArchiveCommand, HistoryCommand},
    msg_info,
    ui::{view_archive, view_search},
};

pub fn handle_history_command(command: HistoryCommand, paths: &DataPaths) -> Result<()> {
    let mut codexi = FileManagement::load_current_state(paths)?;
    match command {
        HistoryCommand::Init { date, initial_amount } => {
            let date = parse_date(&date)?;
            let initial_amount_d = parse_decimal(&initial_amount, "initial_amount")?;
            let account = codexi.get_current_account_mut()?;
            account.initialize(date, initial_amount_d)?;
            FileManagement::save_current_state(&codexi, paths)?;
            msg_info!(
                "codexi initialized with a balance of {:.2} on {}.",
                initial_amount_d,
                date
            );
        }
        HistoryCommand::Adjust { date, physical_amount } => {
            let physical_amount_d = parse_decimal(&physical_amount, "physical_amount")?;
            let date = parse_date(&date)?;
            let account = codexi.get_current_account_mut()?;
            account.adjust_balance(date, physical_amount_d)?;
            FileManagement::save_current_state(&codexi, paths)?;
            msg_info!("Adjust done: {} {}", date, physical_amount_d);
        }
        HistoryCommand::Close { date, description } => {
            let date = parse_date(&date)?;
            let description = parse_text(description);
            let account = codexi.get_current_account_mut()?;
            account.checkpoint(date, description, paths)?;
            FileManagement::save_current_state(&codexi, paths)?;
            msg_info!("Close period of the {} completed", date);
        }
        HistoryCommand::Void { id } => {
            let account = codexi.get_current_account()?;
            let op_id = resolve_id::<Operation, AccountError>(&id, &account.operations)?;
            codexi.void_from_current(op_id)?;
            FileManagement::save_current_state(&codexi, paths)?;
            msg_info!("Operation {} voided.", op_id);
        }
        HistoryCommand::Archive(archive) => match archive.command {
            ArchiveCommand::List => {
                let account = codexi.get_current_account()?;
                let results = FileManagement::list_archive(paths, account.id)?;
                view_archive(&results);
            }
            ArchiveCommand::View { id, date } => {
                let account = codexi.get_current_account()?;

                let id = parse_id(&id)?;
                let date = parse_date(&date)?;
                let account_archive = FileManagement::load_archive(id, date, paths)?;
                let params = SearchParamsBuilder::default().build()?;

                let s_ops = search(&account_archive, &params)?;
                let datas = SearchOperationCollection::build(&codexi, account, &s_ops);
                view_search(&datas);
            }
        },
    }
    Ok(())
}
