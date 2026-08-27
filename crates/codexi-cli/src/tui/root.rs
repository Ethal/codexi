use anyhow::{Result, anyhow};
use chrono::{Local, Months};
use inquire::{Select, Text};
use inquire_derive::Selectable;
use std::fmt;

use codexi::{
    core::{DataPaths, format_date, format_id_short, get_documents_dir, parse_id},
    dto::{
        AccountCollection, AccountItem, BalanceItem, CategoryCollection, CategoryStatsCollection,
        CounterpartyCollection, CounterpartyStatsCollection, CounterpartyTreeCollection, DashboardCollection,
        ExchangeRateCollection, MonthlyReport, SearchOperationCollection, StatsCollection, SummaryCollection,
    },
    file_management::FileManagement,
    logic::{
        balance::Balance,
        codexi::Codexi,
        search::{SearchParamsBuilder, search},
    },
    types::DateRange,
};

use crate::ui::{
    format_optional_bank_item, format_optional_currency_item, overview_account, truncate_text, view_account_context,
    view_balance, view_category, view_category_stats, view_counterparty, view_counterparty_stats, view_dashboard,
    view_exchange_rate, view_financial, view_monthly_report, view_search, view_summary, view_tree,
};
use crate::{clear_terminal, msg_info};

/*==================== Helper – Date ====================*/

#[derive(Debug, Copy, Clone, Selectable)]
enum TuiDate {
    All,
    ThisMonth,
    LastMonth,
    ThisYear,
    LastYear,
    Custom,
    Back,
}

impl fmt::Display for TuiDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TuiDate::All => "All",
            TuiDate::ThisMonth => "This month",
            TuiDate::LastMonth => "Last month",
            TuiDate::ThisYear => "This year",
            TuiDate::LastYear => "Last year",
            TuiDate::Custom => "Custom",
            TuiDate::Back => "← Back",
        };
        write!(f, "{}", s)
    }
}

fn tui_date() -> Result<Option<DateRange>> {
    let choice = TuiDate::select("Select a period").prompt()?;
    let date_range = match choice {
        TuiDate::All => Some(DateRange { from: None, to: None }),
        TuiDate::ThisMonth => {
            let month = &format_date(Local::now().date_naive())[0..7];
            Some(DateRange::parse(Some(month), Some(month))?)
        }
        TuiDate::LastMonth => {
            let last_month = Local::now()
                .checked_sub_months(Months::new(1))
                .ok_or(anyhow!("invalid date"))?;
            let last_month_s = &format_date(last_month.date_naive())[0..7];
            Some(DateRange::parse(Some(last_month_s), Some(last_month_s))?)
        }
        TuiDate::ThisYear => {
            let year = &format_date(Local::now().date_naive())[0..4];
            Some(DateRange::parse(Some(year), Some(year))?)
        }
        TuiDate::LastYear => {
            let last_year = Local::now()
                .checked_sub_months(Months::new(12))
                .ok_or(anyhow!("invalid date"))?;
            let last_year_s = &format_date(last_year.date_naive())[0..4];
            Some(DateRange::parse(Some(last_year_s), Some(last_year_s))?)
        }
        TuiDate::Custom => {
            let from = Text::new("From date (YYYY-MM-DD)").with_default("").prompt()?;
            let from_opt = Some(from).filter(|s| !s.is_empty());
            let to = Text::new("To date (YYYY-MM-DD)").with_default("").prompt()?;
            let to_opt = Some(to).filter(|s| !s.is_empty());
            Some(DateRange::parse(from_opt.as_deref(), to_opt.as_deref())?)
        }
        TuiDate::Back => None,
    };

    Ok(date_range)
}

/*==================== Root ====================*/

#[derive(Debug, Copy, Clone, Selectable)]
enum TuiRoot {
    Overview,
    View,
    Report,
    Account,
    Counterparty,
    Category,
    Backup,
    Quit,
}

impl fmt::Display for TuiRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TuiRoot::Overview => "Accounts Overview",
            TuiRoot::View => "Operations",
            TuiRoot::Report => "Report",
            TuiRoot::Account => "Account",
            TuiRoot::Counterparty => "Counterparty",
            TuiRoot::Category => "Category",
            TuiRoot::Backup => "Backup",
            TuiRoot::Quit => "Quit/Exit",
        };
        write!(f, "{}", s)
    }
}

pub fn tui_root(paths: &DataPaths) -> Result<()> {
    let mut codexi = FileManagement::load_current_state(paths)?;

    loop {
        clear_terminal();
        let title = format!(
            "Codexi - Current account: {}",
            codexi.get_current_account()?.name.clone()
        );
        let choice = TuiRoot::select(&title).with_page_size(10).prompt()?;

        match choice {
            TuiRoot::Overview => {
                let accounts = AccountCollection::build(&codexi);
                overview_account(&accounts);
                Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
            }
            TuiRoot::View => {
                if let Some(date_range) = tui_date()? {
                    let account = codexi.get_current_account()?;
                    let params = SearchParamsBuilder::default()
                        .from(date_range.from)
                        .to(date_range.to)
                        .build()?;
                    let s_ops = search(account, &params)?;
                    let items = SearchOperationCollection::build(&codexi, account, &s_ops);
                    view_search(&items);
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }
            TuiRoot::Report => tui_report(&codexi)?,
            TuiRoot::Account => tui_account(paths, &mut codexi)?,
            TuiRoot::Counterparty => tui_counterparty(&codexi)?,
            TuiRoot::Category => tui_category(&codexi)?,
            TuiRoot::Backup => {
                let default_dir = get_documents_dir()?.to_string_lossy().into_owned();
                let target_dir = Text::new("Target directory").with_default(&default_dir).prompt()?;
                let target_dir_opt = Some(target_dir).filter(|s| !s.is_empty());
                let backup_file = FileManagement::create_backup(paths, target_dir_opt.as_deref())?;
                msg_info!("Backup completed to: {}", backup_file.display());
                Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
            }

            TuiRoot::Quit => break,
        }
    }

    Ok(())
}

/*==================== Report ====================*/

#[derive(Debug, Copy, Clone, Selectable)]
enum TuiReport {
    Dashboard,
    Monthly,
    Counterparty,
    Category,
    Rate,
    Tree,
    Financial,
    Balance,
    Summary,
    Back,
}

impl fmt::Display for TuiReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TuiReport::Dashboard => "Dashborad",
            TuiReport::Monthly => "Monthly",
            TuiReport::Counterparty => "Counterparty",
            TuiReport::Category => "Category",
            TuiReport::Rate => "Rate",
            TuiReport::Tree => "Tree Counterparty - Category",
            TuiReport::Financial => "Financial",
            TuiReport::Balance => "Balance",
            TuiReport::Summary => "Summary",
            TuiReport::Back => "← Back",
        };
        write!(f, "{}", s)
    }
}

pub fn tui_report(codexi: &Codexi) -> Result<()> {
    loop {
        clear_terminal();

        let choice = TuiReport::select("Report").with_page_size(10).prompt()?;

        match choice {
            TuiReport::Dashboard => {
                if let Some(date_range) = tui_date()? {
                    let account = codexi.get_current_account()?;
                    let params = SearchParamsBuilder::default()
                        .from(date_range.from)
                        .to(date_range.to)
                        .build()?;
                    let s_ops = search(account, &params)?;

                    let cp_groups = s_ops.group_by_counterparty(&codexi.counterparties);
                    let cat_groups = s_ops.group_by_category(&codexi.categories);

                    let stats = StatsCollection::build(codexi, account, &s_ops);
                    let cp_stats = CounterpartyStatsCollection::build(cp_groups);
                    let cat_stats = CategoryStatsCollection::build(cat_groups);
                    let dashboard = DashboardCollection::build(stats, cp_stats, cat_stats);
                    view_dashboard(&dashboard);
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }

            TuiReport::Monthly => {
                let account = codexi.get_current_account()?;

                let all_ops = search(account, &SearchParamsBuilder::default().build()?)?;
                let range = DateRange::compute(&all_ops, None, None);

                let from = range.from.ok_or(anyhow!("from required"))?;
                let to = range.to.ok_or(anyhow!("to required"))?;

                let months = DateRange::month_periods(from, to);

                let mut items = Vec::new();
                for (start, end, label) in months {
                    let params = SearchParamsBuilder::default().from(Some(start)).to(Some(end)).build()?;

                    let ops = search(account, &params)?;
                    let stats = StatsCollection::build(codexi, account, &ops);

                    items.push((label, stats));
                }

                let report = MonthlyReport::build(items);
                view_monthly_report(&report);
                Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
            }
            TuiReport::Counterparty => {
                if let Some(date_range) = tui_date()? {
                    let account = codexi.get_current_account()?;
                    let params = SearchParamsBuilder::default()
                        .from(date_range.from)
                        .to(date_range.to)
                        .build()?;
                    let s_ops = search(account, &params)?;
                    let groups = s_ops.group_by_counterparty(&codexi.counterparties);
                    let stats = CounterpartyStatsCollection::build(groups);
                    view_counterparty_stats(&stats);
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }
            TuiReport::Category => {
                if let Some(date_range) = tui_date()? {
                    let account = codexi.get_current_account()?;
                    let params = SearchParamsBuilder::default()
                        .from(date_range.from)
                        .to(date_range.to)
                        .build()?;
                    let s_ops = search(account, &params)?;
                    let groups = s_ops.group_by_category(&codexi.categories);
                    let stats = CategoryStatsCollection::build(groups);
                    view_category_stats(&stats);
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }
            TuiReport::Tree => {
                if let Some(date_range) = tui_date()? {
                    let account = codexi.get_current_account()?;
                    let params = SearchParamsBuilder::default()
                        .from(date_range.from)
                        .to(date_range.to)
                        .build()?;
                    let s_ops = search(account, &params)?;
                    let groups = s_ops.group_by_counterparty_category(&codexi.counterparties, &codexi.categories);
                    let tree = CounterpartyTreeCollection::build(groups);
                    view_tree(&tree);
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }
            TuiReport::Rate => {
                if let Some(date_range) = tui_date()? {
                    let account = codexi.get_current_account()?;
                    let params = SearchParamsBuilder::default()
                        .from(date_range.from)
                        .to(date_range.to)
                        .build()?;
                    let s_ops = search(account, &params)?;
                    let rate_report = ExchangeRateCollection::build(codexi, account, &s_ops);
                    view_exchange_rate(&rate_report);
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }
            TuiReport::Financial => {
                if let Some(date_range) = tui_date()? {
                    let account = codexi.get_current_account()?;
                    let params = SearchParamsBuilder::default()
                        .from(date_range.from)
                        .to(date_range.to)
                        .build()?;
                    let ops = search(account, &params)?;
                    let stats = StatsCollection::build(codexi, account, &ops);

                    view_financial(&stats);
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }
            TuiReport::Balance => {
                if let Some(date_range) = tui_date()? {
                    let account = codexi.get_current_account()?;
                    let params = SearchParamsBuilder::default()
                        .from(date_range.from)
                        .to(date_range.to)
                        .build()?;
                    let balance_items = search(account, &params)?;
                    let balance = BalanceItem::from(Balance::build(&balance_items));
                    view_balance(&balance);
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }
            TuiReport::Summary => {
                let account = codexi.get_current_account()?;
                let params = SearchParamsBuilder::default().build()?;
                let s_ops = search(account, &params)?;
                let summary = SummaryCollection::summary_entry(account, &s_ops);
                view_summary(&summary);
                Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
            }

            TuiReport::Back => break,
        }
    }

    Ok(())
}

/*==================== Account ====================*/

#[derive(Debug, Copy, Clone, Selectable)]
enum TuiAccount {
    Use,
    Context,
    Back,
}
impl fmt::Display for TuiAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TuiAccount::Use => "Use",
            TuiAccount::Context => "Context",
            TuiAccount::Back => "← Back",
        };
        write!(f, "{}", s)
    }
}

struct TuiChoice {
    id: String,
    label: String,
}

impl fmt::Display for TuiChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

fn select_account(items: &AccountCollection) -> Result<Option<String>> {
    let mut options = Vec::new();

    for acc in &items.items {
        let marker = match (acc.current, acc.close) {
            (false, false) => "      ".to_string(),
            (true, false) => "   (*)".to_string(),
            (false, true) => "(c)   ".to_string(),
            (true, true) => "(c)(*)".to_string(),
        };

        let label = format!(
            "{} {} {} - {:<10} {:<7} {} {}",
            marker,
            acc.id,
            format_id_short(&acc.id),
            truncate_text(&acc.name, 10),
            acc.context.account_type,
            format_optional_currency_item(&acc.currency),
            format_optional_bank_item(&acc.bank),
        );

        options.push(TuiChoice {
            id: acc.id.clone(),
            label,
        });
    }

    let back = TuiChoice {
        id: "".to_string(),
        label: "← Back".into(),
    };
    options.push(back);

    let choice = Select::new("Select", options).prompt()?;

    if choice.label == "← Back" {
        return Ok(None);
    }

    Ok(Some(choice.id))
}

pub fn tui_account(paths: &DataPaths, codexi: &mut Codexi) -> Result<()> {
    loop {
        clear_terminal();

        let choice = TuiAccount::select("Account").prompt()?;

        match choice {
            TuiAccount::Use => {
                let items = AccountCollection::build(codexi);
                if let Some(account_id) = select_account(&items)? {
                    let id = parse_id(&account_id)?;
                    codexi.set_current_account(&id)?;
                    FileManagement::save_current_state(codexi, paths)?;
                    Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
                }
            }
            TuiAccount::Context => {
                let codexi = FileManagement::load_current_state(paths)?;
                let account = codexi.get_current_account()?;
                let account_item = AccountItem::build(&codexi, account);
                view_account_context(&account_item);
                Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
            }

            TuiAccount::Back => break,
        }
    }

    Ok(())
}

/*==================== Counterparty ====================*/

#[derive(Debug, Copy, Clone, Selectable)]
enum TuiCounterparty {
    List,
    Back,
}

impl fmt::Display for TuiCounterparty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TuiCounterparty::List => "List",
            TuiCounterparty::Back => "Back",
        };
        write!(f, "{}", s)
    }
}

pub fn tui_counterparty(codexi: &Codexi) -> Result<()> {
    loop {
        clear_terminal();

        let choice = TuiCounterparty::select("Counterparty").prompt()?;

        match choice {
            TuiCounterparty::List => {
                let items = CounterpartyCollection::build(&codexi.counterparties);
                view_counterparty(&items);
                Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
            }

            TuiCounterparty::Back => break,
        }
    }

    Ok(())
}

/*==================== Category ====================*/

#[derive(Debug, Copy, Clone, Selectable)]
enum TuiCategory {
    List,
    Back,
}

impl fmt::Display for TuiCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TuiCategory::List => "List",
            TuiCategory::Back => "Back",
        };
        write!(f, "{}", s)
    }
}

pub fn tui_category(codexi: &Codexi) -> Result<()> {
    loop {
        clear_terminal();

        let choice = TuiCategory::select("Category").prompt()?;

        match choice {
            TuiCategory::List => {
                let items = CategoryCollection::build(&codexi.categories);
                view_category(&items);
                Text::new("Press Enter to continue").with_placeholder("...").prompt()?;
            }

            TuiCategory::Back => break,
        }
    }

    Ok(())
}
