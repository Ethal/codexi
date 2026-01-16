// src/core/wammet/viewer.rs

use thousands::Separable;

use super::codexi::Codexi;
use super::codexi::SearchItem;
use super::codexi::BalanceResult;
use super::codexi::ResumeResult;

/// Methods for viewing codexi data
impl Codexi {
    /// view snapshot
    pub fn view_snapshot(datas: &[String]) {
        println!("┌-----------------------------┐");
        println!("|Snapshot(s)                  |");
        println!("├-----------------------------┤");
        for f in datas {
            println!("|{} |", f);
        }
        println!("└-----------------------------┘");
    }
    /// view archive
    pub fn view_archive(datas: &[String]) {
        println!("┌-----------------------------┐");
        println!("|Archive(s)                   |");
        println!("├-----------------------------┤");
        for f in datas {
            println!("|{}      |", f);
        }
        println!("└-----------------------------┘");
    }
    /// view balance
    pub fn view_balance(balance: &BalanceResult) {
        println!("┌---------------------------┐");
        println!("|codexi balance summary     |");
        println!("├--------┌------------------┤");
        println!("|Credit  |{:>18}|", format!("{:.2}", balance.credit).separate_with_commas());
        println!("|Debit   |{:>18}|", format!("{:.2}", balance.debit).separate_with_commas());
        println!("|Balance |{:>18}|", format!("{:.2}", balance.total).separate_with_commas());
        println!("└--------└------------------┘");
    }
    /// view search results
    pub fn view_search(rows: &[SearchItem]){
        println!("┌-----------------------------------------------------------------------------------------------┐");
        println!("|{:<95}|", "Operation(s)");
        println!("├-----------------------------------------------------------------------------------------------┤");
        println!("|Index  |Date      | Type  |           Montant|           Balance|Description                   |");
        println!("├-------|----------|-------|------------------|------------------|------------------------------┤");

        for item in rows {
            println!(
                "|#{:<6}|{}|{}|{:>18}|{:>18}|{:<30}|",
                item.index,
                item.op.date,
                item.op.flow,
                format!("{:.2}", item.op.amount).separate_with_commas(),
                format!("{:.2}", item.balance).separate_with_commas(),
                Self::truncate_desc(&item.op.description, 30),
            );
        }

        println!("└-------└----------└-------└------------------└------------------└------------------------------┘");
        println!();
        println!("Total operations found: {}", rows.len());
        println!();
        println!("Note: Descriptions longer than 30 characters are truncated with '...'.");
        println!("Remember to regularly perform closing operations to maintain accurate financial records.");
        println!();
    }
    /// view resume
    pub fn view_resume(resume: &ResumeResult) {

        println!("┌-------------------------------------------------------------------------------┐");
        println!("|codexi resume                                                                  |");
        println!("├----------------------┬--------------------------------------------------------┤");
        println!("|number of transactions|{:>18}| latest date transations: {:>10} |", resume.current_nb_transaction, resume.latest_transaction_date);
        println!("|number of init        |{:>18}| latest date init:        {:>10} |", resume.current_nb_init, resume.latest_init_date);
        println!("|number of adjustments |{:>18}| latest date adjustment:  {:>10} |", resume.current_nb_adjust, resume.latest_adjust_date);
        println!("|number of closings    |{:>18}| latest date closing:     {:>10} |", resume.current_nb_close, resume.latest_close_date);
        println!("|total operations      |{:>18}|                                     |", resume.current_nb_transaction+resume.current_nb_init+resume.current_nb_adjust+resume.current_nb_close);
        println!("|current balance       |{:>18}|                                     |", format!("{:.2}", resume.current_balance).separate_with_commas());
        println!("└----------------------└------└-------------------------------------------------┘");
        println!();
        println!("Note: 'latest date' corresponds to the most recent date for each operation type.");
        println!("Remember to regularly perform closing operations to maintain accurate financial records.");
        println!();
    }
    /// Truncate description for display
    fn truncate_desc(desc: &str, max_width: usize) -> String {
        // If the visible length is already OK → simple formatting
        if desc.chars().count() <= max_width {
            return format!("{:<width$}", desc, width = max_width);
        }

        // Otherwise → truncate without ever breaking a UTF-8 character
        let visible = max_width.saturating_sub(3);

        let truncated: String = desc.chars().take(visible).collect();

        format!("{:<width$}", format!("{}...", truncated), width = max_width)
    }

}
