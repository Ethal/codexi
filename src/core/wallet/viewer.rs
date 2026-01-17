// src/core/wallet/viewer.rs

use thousands::Separable;
use owo_colors::{OwoColorize, Style};

use super::codexi::Codexi;
use super::codexi::SearchItem;
use super::codexi::BalanceResult;
use super::codexi::ResumeResult;
use super::operation_flow::OperationFlow;

/// Methods for viewing codexi data
impl Codexi {
    /// view to list the snapshot file
    pub fn view_snapshot(datas: &[String]) {
        println!("┌─────────────────────────────┐");
        let title_text = format!("{:<28}", "Snapshot(s)");
        println!("│ {}│", title_text.cyan().bold());
        println!("├─────────────────────────────┤");
        if datas.len() == 0 {
            println!("│ {:<28}│", "No snapshot");
        } else {
            for f in datas {
                println!("│ {:<28}│", f);
            }
        }
        println!("└─────────────────────────────┘");
    }
    /// view to list the archive file
    pub fn view_archive(datas: &[String]) {
        println!("┌─────────────────────────────┐");
        let title_text = format!("{:<28}", "Archive(s)");
        println!("│ {}│", title_text.cyan().bold());
        println!("├─────────────────────────────┤");
        if datas.len() == 0 {
            println!("│ {:<28}│", "No archive");
        } else {
            for f in datas {
                println!("│ {:<28}│", f);
            }
        }
        println!("└─────────────────────────────┘");
    }
    /// view the balance (credit/debit/balance)
    pub fn view_balance(balance: &BalanceResult) {
        println!("┌───────────────────────────┐");
        println!("│ {}    │", "codexi balance summary".cyan().bold());
        println!("├────────┬──────────────────┤");
        println!("│Credit  │{:>18}│", format!("{:.2}", balance.credit).separate_with_commas().green());
        println!("│Debit   │{:>18}│", format!("{:.2}", balance.debit).separate_with_commas().red());
        println!("│Balance │{:>18}│", format!("{:.2}", balance.total).separate_with_commas().yellow().bold());
        println!("└────────┴──────────────────┘");
    }
    /// view of the search results
    pub fn view_search(rows: &[SearchItem]){
        println!("┌───────────────────────────────────────────────────────────────────────────────────────────────┐");
        let title_text = format!("{:<94}", "Operation(s)");
        println!("│ {}│", title_text.bold().cyan());
        println!("├───────┬──────────┬───────┬──────────────────┬──────────────────┬──────────────────────────────┤");
        println!("│Index  │Date      │ Type  │           Montant│           Balance│Description                   │");
        println!("├───────┼──────────┼───────┼──────────────────┼──────────────────┼──────────────────────────────┤");

        for item in rows {
            // Determine the color according to the flow (credit/debit)
            let amount_str = format!("{:.2}", item.op.amount).separate_with_commas();
            let amount_style = match item.op.flow {
                OperationFlow::Credit => Style::new().green(),
                OperationFlow::Debit  => Style::new().red(),
                OperationFlow::None   => Style::new().dimmed(),
            };
            let colored_amount = amount_str.style(amount_style);

            let index_style = Style::new().dimmed();
            let index_str = format!("#{}", item.index);
            let colored_index = index_str.style(index_style);

            println!(
                "│{:<7}│{}│{}│{:>18}│{:>18}│{:<30}│",
                colored_index,
                item.op.date,
                item.op.flow,
                colored_amount,
                format!("{:.2}", item.balance).separate_with_commas().yellow(),
                Self::truncate_desc(&item.op.description, 30),
            );
        }

        let note_style = Style::new().blue().italic();

        println!("└───────┴──────────┴───────┴──────────────────┴──────────────────┴──────────────────────────────┘");
        println!();
        println!("Total operations found: {}", rows.len());
        println!();
        println!("{}", "Note: Descriptions longer than 30 characters are truncated with '...'.".style(note_style));
        println!("{}", "Remember to regularly perform closing operations to maintain accurate financial records.".style(note_style));
        println!();
    }
    /// view to resume the codexi
    pub fn view_resume(resume: &ResumeResult) {

        let title_style = Style::new().cyan().bold();
        let label_style = Style::new().dimmed();
        let value_style = Style::new().yellow();
        let note_style = Style::new().blue().italic();

        println!("┌────────────────────────────────────────────────────────────────────────────────┐");
        let title_text = format!("{:<79}", "codexi resume");
        println!("│ {}│", title_text.style(title_style));
        println!("├──────────────────────┬──────────────────┬──────────────────────────────────────┤");
        println!("│{:<22}│{:>18}│ latest date transactions: {:>10} │",
                "number of transactions".style(label_style),
                resume.current_nb_transaction,
                resume.latest_transaction_date.style(value_style));

        println!("│{:<22}│{:>18}│ latest date init: {:>18} │",
                "number of init".style(label_style),
                resume.current_nb_init,
                resume.latest_init_date.style(value_style));

        println!("│{:<22}│{:>18}│ latest date adjustment: {:>12} │",
                "number of adjustments".style(label_style),
                resume.current_nb_adjust,
                resume.latest_adjust_date.style(value_style));

        println!("│{:<22}│{:>18}│ latest date closing: {:>15} │",
                "number of closings ".style(label_style),
                resume.current_nb_close,
                resume.latest_close_date.style(value_style));

        println!("│{:<22}│{:>18}│                                      │",
            "total operations".style(label_style),
            resume.current_nb_op.style(value_style).bold());

        println!("│{:<22}│{:>18}│                                      │",
            "current balance".style(label_style),
            format!("{:.2}", resume.current_balance).separate_with_commas().style(value_style).bold());

        println!("└──────────────────────┴──────────────────┴──────────────────────────────────────┘");
        println!();
        println!("{}", "Note: 'latest date' corresponds to the most recent date for each operation type.".style(note_style));
        println!("{}", "Remember to regularly perform closing operations to maintain accurate financial records.".style(note_style));
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
