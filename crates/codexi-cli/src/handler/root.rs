// src/handler/main.rs

use anyhow::Result;
use chrono::Local;
use std::path::Path;

use codexi::{
    core::{
        DataPaths, format_date, normalize_string, normalize_vec_input, parse_date, parse_decimal,
        parse_optional_decimal, parse_text,
    },
    dto::{AccountCollection, SearchOperationCollection, StatementCollection},
    file_management::FileManagement,
    logic::{
        account::Account,
        category::{Category, CategoryError},
        codexi::CodexiError,
        counterparty::{Counterparty, CounterpartyError},
        operation::{OperationFlow, OperationKind, RegularKind},
        search::{NulidSearchFilter, SearchParamsBuilder, search},
        utils::resolve_by_id_or_name,
    },
    types::DateRange,
};

use crate::{
    command::{Cli, RootCommand},
    export::export_statement_html,
    handler::{
        account::handle_account_command, admin::handle_admin_command, bank::handle_bank_command,
        category::handle_category_command, counterparty::handle_counterparty_command,
        currency::handle_currency_command, data::handle_data_command, history::handle_history_command,
        loan::handle_loan_command, operation::handle_operation_command, report::handle_report_command,
        settings::handle_settings_command,
    },
    msg_info, msg_warn,
    ui::overview_account,
};

use crate::ui::view_search;

pub fn handle_root_command(cli: Cli, paths: &DataPaths, cwd: &Path) -> Result<()> {
    let skip_confirm = cli.yes;
    let command = cli.command.unwrap();

    match command {
        RootCommand::Overview => {
            let codexi = FileManagement::load_current_state(paths)?;
            let accounts = AccountCollection::build(&codexi);
            overview_account(&accounts);
        }
        RootCommand::Use { id } => {
            let mut codexi = FileManagement::load_current_state(paths)?;
            let id_n = resolve_by_id_or_name::<Account, CodexiError>(&id, &codexi.accounts)?;
            codexi.set_current_account(&id_n)?;
            FileManagement::save_current_state(&codexi, paths)?;
            let account = codexi.get_account_by_id(&id_n)?;
            msg_info!("Switched to account: {} ({})", account.name, id_n);
        }

        RootCommand::Debit {
            date,
            amount,
            description,
            counterparty,
            category,
        } => {
            let date = parse_date(&date)?;
            let amount_s = normalize_string(&amount);
            let amount_d = parse_decimal(&amount_s, "amount")?;
            let desc = normalize_vec_input(description);
            let desc_s = parse_text(desc);

            let mut codexi = FileManagement::load_current_state(paths)?;
            let category_id = category
                .map(|name| resolve_by_id_or_name::<Category, CategoryError>(&name, &codexi.categories.list))
                .transpose()?;
            let counterparty_id = counterparty
                .map(|name| {
                    resolve_by_id_or_name::<Counterparty, CounterpartyError>(&name, &codexi.counterparties.list)
                })
                .transpose()?;
            let account = codexi.get_current_account_mut()?;

            account.register_transaction(
                date,
                OperationKind::Regular(RegularKind::Transaction),
                OperationFlow::Debit,
                amount_d,
                desc_s.clone(),
                counterparty_id,
                category_id,
            )?;
            FileManagement::save_current_state(&codexi, paths)?;
            msg_info!("Debit operation added: {} {} {}", date, amount_d, desc_s);
        }

        RootCommand::Credit {
            date,
            amount,
            description,
            counterparty,
            category,
        } => {
            let date = parse_date(&date)?;
            let amount_s = normalize_string(&amount);
            let amount_d = parse_decimal(&amount_s, "amount")?;
            let desc = normalize_vec_input(description);
            let desc_s = parse_text(desc);

            let mut codexi = FileManagement::load_current_state(paths)?;
            let category_id = category
                .map(|name| resolve_by_id_or_name::<Category, CategoryError>(&name, &codexi.categories.list))
                .transpose()?;
            let counterparty_id = counterparty
                .map(|name| {
                    resolve_by_id_or_name::<Counterparty, CounterpartyError>(&name, &codexi.counterparties.list)
                })
                .transpose()?;
            let account = codexi.get_current_account_mut()?;

            let reg_kind = RegularKind::Transaction;
            account.register_transaction(
                date,
                OperationKind::Regular(reg_kind),
                OperationFlow::Credit,
                amount_d,
                desc_s.clone(),
                counterparty_id,
                category_id,
            )?;

            FileManagement::save_current_state(&codexi, paths)?;
            msg_info!("Crebit operation added: {} {} {}", date, amount_d, desc_s);
        }
        RootCommand::Interest {
            date,
            amount,
            description,
            counterparty,
            category,
        } => {
            let date = parse_date(&date)?;
            let amount_s = normalize_string(&amount);
            let amount_d = parse_decimal(&amount_s, "amount")?;
            let desc = normalize_vec_input(description);
            let desc_s = parse_text(desc);

            let mut codexi = FileManagement::load_current_state(paths)?;
            let category_id = category
                .map(|name| resolve_by_id_or_name::<Category, CategoryError>(&name, &codexi.categories.list))
                .transpose()?;
            let counterparty_id = counterparty
                .map(|name| {
                    resolve_by_id_or_name::<Counterparty, CounterpartyError>(&name, &codexi.counterparties.list)
                })
                .transpose()?;
            let account = codexi.get_current_account_mut()?;

            let reg_kind = RegularKind::Interest;
            account.register_transaction(
                date,
                OperationKind::Regular(reg_kind),
                OperationFlow::Credit,
                amount_d,
                desc_s.clone(),
                counterparty_id,
                category_id,
            )?;

            FileManagement::save_current_state(&codexi, paths)?;
            msg_info!("Crebit operation added: {} {} {}", date, amount_d, desc_s);
        }
        RootCommand::Transfer {
            date,
            amount_from,
            amount_to,
            account_id_to,
            description,
            category,
        } => {
            let date = parse_date(&date)?;
            let amount_from_s = normalize_string(&amount_from);
            let amount_from_d = parse_decimal(&amount_from_s, "amount_from")?;
            let amount_to_s = normalize_string(&amount_to);
            let amount_to_d = parse_decimal(&amount_to_s, "amount_to")?;
            let desc = normalize_vec_input(description);
            let desc_s = parse_text(desc);

            let mut codexi = FileManagement::load_current_state(paths)?;
            let acc_id_to = resolve_by_id_or_name::<Account, CodexiError>(&account_id_to, &codexi.accounts)?;
            let category_id = category
                .map(|name| resolve_by_id_or_name::<Category, CategoryError>(&name, &codexi.categories.list))
                .transpose()?;

            codexi.transfer(date, amount_from_d, amount_to_d, acc_id_to, desc_s.clone(), category_id)?;

            FileManagement::save_current_state(&codexi, paths)?;
            msg_info!(
                "Transfer operation added: {} {} {} to {} {}",
                date,
                amount_from_d,
                amount_to_d,
                acc_id_to,
                desc_s
            );
        }

        RootCommand::Search {
            from,
            to,
            text,
            kind,
            flow,
            counterparty,
            category,
            amount_min,
            amount_max,
            last,
            today,
            open,
        } => {
            let codexi = FileManagement::load_current_state(paths)?;
            let g_filter = match category.as_deref() {
                None => NulidSearchFilter::Any,
                Some("None") => NulidSearchFilter::NoneOnly,
                Some(v) => NulidSearchFilter::One(resolve_by_id_or_name::<Category, CategoryError>(
                    v,
                    &codexi.categories.list,
                )?),
            };
            let c_filter = match counterparty.as_deref() {
                None => NulidSearchFilter::Any,
                Some("None") => NulidSearchFilter::NoneOnly,
                Some(v) => NulidSearchFilter::One(resolve_by_id_or_name::<Counterparty, CounterpartyError>(
                    v,
                    &codexi.counterparties.list,
                )?),
            };
            let amount_min_d = parse_optional_decimal(&amount_min, "amount_min")?;
            let amount_max_d = parse_optional_decimal(&amount_max, "amount_max")?;
            let mut range = DateRange::parse(from.as_deref(), to.as_deref())?;
            if today {
                let from = format_date(Local::now().date_naive());
                let to = format_date(Local::now().date_naive());
                range = DateRange::parse(Some(from.as_ref()), Some(to.as_ref()))?;
            }

            let account = codexi.get_current_account()?;
            let params = SearchParamsBuilder::default()
                .from(range.from)
                .to(range.to)
                .text(text)
                .kind(kind)
                .flow(flow)
                .counterparty(c_filter)
                .category(g_filter)
                .amount_min(amount_min_d)
                .amount_max(amount_max_d)
                .latest(last)
                .build()?;

            let s_ops = search(account, &params)?;
            let search_items = SearchOperationCollection::build(&codexi, account, &s_ops);
            if search_items.items.is_empty() {
                msg_warn!("No data available as per criteria.");
                return Ok(());
            }
            if open {
                let statement_results = StatementCollection::build(&codexi, account, &s_ops);
                let html = export_statement_html(statement_results)?;
                let file_path = FileManagement::export_html(&html, cwd)?;
                msg_info!("search completed (report.html)");
                opener::open_browser(file_path)?;
            } else {
                view_search(&search_items);
            }
        }
        RootCommand::Settings(args) => handle_settings_command(args.command, paths)?,
        RootCommand::Operation(args) => handle_operation_command(args.command, paths)?,
        RootCommand::Report(args) => handle_report_command(args.command, cwd, paths)?,
        RootCommand::Data(args) => handle_data_command(args.command, cwd, paths, skip_confirm)?,
        RootCommand::History(args) => handle_history_command(args.command, paths)?,
        RootCommand::Admin(args) => handle_admin_command(args.command, cwd, paths, skip_confirm)?,
        RootCommand::Account(args) => handle_account_command(args.command, paths)?,
        RootCommand::Bank(args) => handle_bank_command(args.command, paths)?,
        RootCommand::Currency(args) => handle_currency_command(args.command, paths)?,
        RootCommand::Counterparty(args) => handle_counterparty_command(args.command, paths)?,
        RootCommand::Category(args) => handle_category_command(args.command, paths)?,
        RootCommand::Loan(args) => handle_loan_command(args.command, paths)?,
    }
    Ok(())
}
