// src/handler/report.rs

use anyhow::{Result, anyhow};
use rust_decimal::Decimal;
use std::path::Path;

use codexi::{
    core::DataPaths,
    dto::{
        BalanceItem, CategoryStatsCollection, CounterpartyStatsCollection, CounterpartyTreeCollection,
        DashboardCollection, ExchangeRateCollection, MonthlyReport, StatementCollection, StatsCollection,
        SummaryCollection,
    },
    file_management::FileManagement,
    logic::{
        balance::Balance,
        search::{SearchParamsBuilder, search},
    },
    types::DateRange,
};

use crate::{
    command::ReportCommand,
    export::{export_statement_html, export_stats_html},
    msg_info, msg_warn,
    ui::{
        view_balance, view_category_stats, view_counterparty_stats, view_dashboard, view_exchange_rate, view_financial,
        view_monthly_report, view_summary, view_tree,
    },
};

pub fn handle_report_command(command: ReportCommand, cwd: &Path, paths: &DataPaths) -> Result<()> {
    let codexi = FileManagement::load_current_state(paths)?;
    let account = codexi.get_current_account()?;
    match command {
        ReportCommand::Balance { from, to } => {
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let params = SearchParamsBuilder::default().from(range.from).to(range.to).build()?;

            let balance_items = search(account, &params)?;
            let balance = BalanceItem::from(Balance::build(&balance_items));
            if balance.total == Decimal::ZERO && balance.credit == Decimal::ZERO && balance.debit == Decimal::ZERO {
                msg_warn!("No data available");
            } else {
                view_balance(&balance);
            }
        }
        ReportCommand::Dashboard { from, to } => {
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let params = SearchParamsBuilder::default().from(range.from).to(range.to).build()?;
            let s_ops = search(account, &params)?;

            let cp_groups = s_ops.group_by_counterparty(&codexi.counterparties);
            let cat_groups = s_ops.group_by_category(&codexi.categories);

            let stats = StatsCollection::build(&codexi, account, &s_ops);
            let cp_stats = CounterpartyStatsCollection::build(cp_groups);
            let cat_stats = CategoryStatsCollection::build(cat_groups);
            let dashboard = DashboardCollection::build(stats, cp_stats, cat_stats);
            view_dashboard(&dashboard);
        }
        ReportCommand::Counterparty { from, to } => {
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let params = SearchParamsBuilder::default().from(range.from).to(range.to).build()?;
            let s_ops = search(account, &params)?;
            let groups = s_ops.group_by_counterparty(&codexi.counterparties);
            let stats = CounterpartyStatsCollection::build(groups);
            view_counterparty_stats(&stats);
        }
        ReportCommand::Category { from, to } => {
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let params = SearchParamsBuilder::default().from(range.from).to(range.to).build()?;
            let s_ops = search(account, &params)?;
            let groups = s_ops.group_by_category(&codexi.categories);
            let stats = CategoryStatsCollection::build(groups);
            view_category_stats(&stats);
        }
        ReportCommand::Rate { from, to } => {
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let params = SearchParamsBuilder::default().from(range.from).to(range.to).build()?;
            let s_ops = search(account, &params)?;
            if s_ops.is_empty_active() {
                msg_warn!("No data available");
                return Ok(());
            }
            let rate_report = ExchangeRateCollection::build(&codexi, account, &s_ops);
            view_exchange_rate(&rate_report);
        }
        ReportCommand::Tree { from, to } => {
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let params = SearchParamsBuilder::default().from(range.from).to(range.to).build()?;
            let s_ops = search(account, &params)?;
            if s_ops.is_empty_active() {
                msg_warn!("No data available");
                return Ok(());
            }
            let groups = s_ops.group_by_counterparty_category(&codexi.counterparties, &codexi.categories);
            let tree = CounterpartyTreeCollection::build(groups);
            view_tree(&tree);
        }
        ReportCommand::Monthly { from, to } => {
            // resolve from/to from the operations if not provide
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let all_ops = search(account, &SearchParamsBuilder::default().build()?)?;
            let range = DateRange::compute(&all_ops, range.from, range.to);

            let from_date = range.from.ok_or(anyhow!("from is required"))?;
            let to_date = range.to.ok_or(anyhow!("to is required"))?;

            let months = DateRange::month_periods(from_date, to_date);

            let mut month_items = Vec::new();
            for (start, end, label) in months {
                let params = SearchParamsBuilder::default().from(Some(start)).to(Some(end)).build()?;
                let s_ops = search(account, &params)?;
                let stats = StatsCollection::build(&codexi, account, &s_ops);
                month_items.push((label, stats));
            }

            let report = MonthlyReport::build(month_items);
            view_monthly_report(&report);
        }
        ReportCommand::Financial { from, to, open } => {
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let params = SearchParamsBuilder::default().from(range.from).to(range.to).build()?;
            let s_ops = search(account, &params)?;
            let stats = StatsCollection::build(&codexi, account, &s_ops);
            if s_ops.is_empty() {
                msg_warn!("No data available");
                return Ok(());
            }
            if open {
                let html = export_stats_html(stats)?;
                let file_path = FileManagement::export_html(&html, cwd)?;
                msg_info!("stats completed (report.html)");
                opener::open_browser(file_path)?;
            } else {
                view_financial(&stats);
            }
        }
        ReportCommand::Summary => {
            let params = SearchParamsBuilder::default().build()?;
            let s_ops = search(account, &params)?;
            let summary = SummaryCollection::summary_entry(account, &s_ops);
            view_summary(&summary);
        }

        ReportCommand::Statement { from, to, open } => {
            let range = DateRange::parse(from.as_deref(), to.as_deref())?;
            let params = SearchParamsBuilder::default().from(range.from).to(range.to).build()?;

            let s_ops = search(account, &params)?;
            let statement_results = StatementCollection::build(&codexi, account, &s_ops);
            if s_ops.is_empty() || statement_results.items.is_empty() {
                msg_warn!("No data available");
                return Ok(());
            }

            let html = export_statement_html(statement_results)?;
            let file_path = FileManagement::export_html(&html, cwd)?;
            msg_info!("statement completed (report.html)");
            if open {
                opener::open_browser(file_path)?;
            }
        }
    }
    Ok(())
}
