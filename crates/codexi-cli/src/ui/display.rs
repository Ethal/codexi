// src/ui.rs
use console::Style;
use rust_decimal::Decimal;
use thousands::Separable;

use codexi::{
    core::{CoreWarning, format_id_short, format_optional_text},
    dto::{DashboardCollection, SearchOperationCollection, SummaryCollection},
};

use crate::ui::{
    CREDIT_STYLE, DEBIT_STYLE, NOTE_STYLE, STYLE_DANGER, STYLE_MUTED, STYLE_NORMAL, TITLE_STYLE, VALUE_STYLE,
    draw_savings_bar, format_ui_left_decimal, truncate_text,
};

pub fn view_warning(warnings: &[CoreWarning]) {
    let title_text = TITLE_STYLE.apply_to("Warning(s)");
    println!("{}", title_text);
    for warn in warnings {
        println!(" {}", warn);
    }
}

/// view to list the snapshot file
pub fn view_snapshot(datas: &[String]) {
    let title_text = TITLE_STYLE.apply_to("Snapshot(s)");
    println!();
    println!("{}", title_text);
    if datas.is_empty() {
        println!(" No snapshot");
    } else {
        for f in datas {
            println!(" {}", f);
        }
    }
    if datas.len() > 5 {
        println!();
        let note_text =
            NOTE_STYLE.apply_to("Note: Considered to performe a `data clean`, snapshot file grether than 5.");
        println!("{}", note_text);
        println!();
    }
}
/// view to list the archive file
pub fn view_archive(datas: &[String]) {
    let title_text = TITLE_STYLE.apply_to("Archive(s)");
    println!();
    println!("{}", title_text);
    if datas.is_empty() {
        println!(" No archive");
    } else {
        for f in datas {
            println!(" {}", f);
        }
    }
    println!();
}

/// view of the search results
pub fn view_search(datas: &SearchOperationCollection) {
    let title_text = TITLE_STYLE.apply_to("Operation(s)");
    println!(
        "┌────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐"
    );
    println!("│ {:<115}│", title_text);
    println!(
        "├───────┬──────────┬───────┬──────────────────┬──────────────────┬───────────────────────────────────────────────────┤"
    );
    println!(
        "│Id     │Date      │ Type  │            Amount│           Balance│Description                                        │"
    );
    println!(
        "├───────┼──────────┼───────┼──────────────────┼──────────────────┼───────────────────────────────────────────────────┤"
    );

    for item in datas.items.iter() {
        let (id_style, row_style) = match item.void_by {
            Some(_) => (STYLE_DANGER, STYLE_MUTED),
            None => (STYLE_MUTED, Style::new()),
        };

        // Determine the color according to the flow (credit/debit)
        let amount_style = match item.void_by {
            Some(_) => STYLE_MUTED,
            None => match item.flow.as_str() {
                "Credit" => CREDIT_STYLE,
                "Debit" => DEBIT_STYLE,
                _ => STYLE_MUTED,
            },
        };
        let balance_style = match item.void_by {
            Some(_) => STYLE_MUTED,
            None => VALUE_STYLE,
        };

        let id_text = id_style.apply_to(format!("#{}", format_id_short(&item.id)));
        let amount_text = amount_style.apply_to(format!("{:.2}", item.amount).separate_with_commas());
        let balance_text = balance_style.apply_to(format!("{:.2}", item.balance).separate_with_commas());

        println!(
            "│{:<7}│{}│{:<7}│{:>18}│{:>18}│{:<51}│",
            id_text,
            row_style.apply_to(&item.date),
            row_style.apply_to(&item.flow),
            amount_text,
            balance_text,
            row_style.apply_to(truncate_text(&item.description, 50)),
        );
    }

    println!(
        "└───────┴──────────┴───────┴──────────────────┴──────────────────┴───────────────────────────────────────────────────┘"
    );
    println!();
    println!("Total operations found: {}", datas.counts.total());
    println!();
    let voided_op = STYLE_DANGER.apply_to("#XXXXX");
    println!("{}", NOTE_STYLE.apply_to("Note:"));
    println!("{} Operation voided", voided_op,);
    println!(
        "{}",
        NOTE_STYLE.apply_to("Descriptions longer than 50 characters are truncated with '...'.")
    );
    println!(
        "{}",
        NOTE_STYLE.apply_to("Remember to regularly perform closing operations to maintain accurate financial records.")
    );
    println!();
}
/// view to a summary of the codexi
pub fn view_summary(summary: &SummaryCollection) {
    let title_text = format!("{:<84}", "Current Acccount summary");
    println!("┌─────────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ {}│", TITLE_STYLE.apply_to(title_text));
    println!("├────────────────────────┬──────────────────┬─────────────────────────────────────────┤");
    println!(
        "│{:<24}│{:>18}│ {} {:>18} │",
        STYLE_NORMAL.apply_to("Regulars count"),
        VALUE_STYLE.apply_to(summary.counts.regular),
        STYLE_NORMAL.apply_to("Latest date regular:"),
        VALUE_STYLE.apply_to(summary.anchors.last_regular.clone().unwrap_or("..........".to_string()))
    );

    println!(
        "│{:<24}│{:>18}│ {} {:>21} │",
        STYLE_NORMAL.apply_to("Init count"),
        VALUE_STYLE.apply_to(summary.counts.init),
        STYLE_NORMAL.apply_to("Latest date init:"),
        VALUE_STYLE.apply_to(summary.anchors.last_init.clone().unwrap_or("..........".to_string())),
    );

    println!(
        "│{:<24}│{:>18}│ {} {:>15} │",
        STYLE_NORMAL.apply_to("Adjustments count"),
        VALUE_STYLE.apply_to(summary.counts.adjust),
        STYLE_NORMAL.apply_to("Latest date adjustment:"),
        VALUE_STYLE.apply_to(summary.anchors.last_adjust.clone().unwrap_or("..........".to_string())),
    );

    println!(
        "│{:<24}│{:>18}│ {} {:>11} │",
        STYLE_NORMAL.apply_to("Void operation count"),
        VALUE_STYLE.apply_to(summary.counts.void),
        STYLE_NORMAL.apply_to("Latest date void operation:"),
        VALUE_STYLE.apply_to(summary.anchors.last_void.clone().unwrap_or("..........".to_string())),
    );

    println!(
        "│{:<24}│{:>18}│ {} {:>18} │",
        STYLE_NORMAL.apply_to("Closings count"),
        VALUE_STYLE.apply_to(summary.counts.checkpoint),
        STYLE_NORMAL.apply_to("Latest date closing:"),
        VALUE_STYLE.apply_to(
            summary
                .anchors
                .last_checkpoint
                .clone()
                .unwrap_or("..........".to_string())
        ),
    );

    println!(
        "│{:<24}│{:>18}│                                         │",
        STYLE_NORMAL.apply_to("Operations count"),
        VALUE_STYLE.apply_to(summary.counts.total()).bold()
    );

    println!(
        "│{:<24}│{:>18}│                                         │",
        STYLE_NORMAL.apply_to("Balance"),
        VALUE_STYLE
            .apply_to(format!("{:.2}", summary.balance.total).separate_with_commas())
            .bold()
    );

    println!("└────────────────────────┴──────────────────┴─────────────────────────────────────────┘");
    println!();
    println!(
        "{}",
        NOTE_STYLE.apply_to("Note: 'latest date' corresponds to the most recent date for each operation type.")
    );
    println!(
        "{}",
        NOTE_STYLE.apply_to("Remember to regularly perform closing operations to maintain accurate financial records.")
    );
    println!();
}

pub fn view_dashboard(d: &DashboardCollection) {
    let savings_line = match d.savings_rate {
        Some(rate) => {
            let rate_str = format!("{:>12.2}%", rate);
            let rate_styled = if rate < Decimal::ZERO {
                DEBIT_STYLE.apply_to(rate_str)
            } else {
                CREDIT_STYLE.apply_to(rate_str)
            };
            format!("  {:<16} {:<26}", STYLE_MUTED.apply_to("Savings rate"), rate_styled,)
        }
        None => format!(
            "  {:<16} {:<26}",
            STYLE_MUTED.apply_to("Savings rate"),
            STYLE_MUTED.apply_to("N/A"),
        ),
    };

    let savings_bar_line = match d.savings_rate {
        Some(rate) => format!("  {:<43}  ", draw_savings_bar(rate, 40)),
        None => format!("  {:<43}", STYLE_MUTED.apply_to("(not supported by account type)"),),
    };

    println!();
    println!(
        "{}",
        TITLE_STYLE.apply_to(format!(
            "Dashboard — {} — {} to {}",
            d.account_name,
            format_optional_text(d.from.as_deref()),
            format_optional_text(d.to.as_deref())
        ))
    );
    println!();

    // ── Section 1 : Financial (gauche) + Top expenses (droite) ──
    let sep = "  │  ";

    // Header
    let h_left = format!("{:<45}", TITLE_STYLE.apply_to("Financial summary"));
    let h_right = format!("{}", TITLE_STYLE.apply_to("Top 5 expenses"));
    println!("{}{}{}", h_left, sep, h_right);
    println!("{}──┼──{}", "─".repeat(45), "─".repeat(52));

    // Lignes financial + top expenses côte à côte
    let fin_lines: Vec<String> = vec![
        format!(
            "  {:<16} {:<26}",
            STYLE_MUTED.apply_to("Credit"),
            CREDIT_STYLE.apply_to(format_ui_left_decimal(d.total_credit, 2)),
        ),
        format!(
            "  {:<16} {:<26}",
            STYLE_MUTED.apply_to("Debit"),
            DEBIT_STYLE.apply_to(format_ui_left_decimal(d.total_debit, 2)),
        ),
        format!(
            "  {:<16} {:<26}",
            STYLE_MUTED.apply_to("Balance"),
            VALUE_STYLE.apply_to(format_ui_left_decimal(d.balance, 2)),
        ),
        format!(
            "  {:<16} ops: {:<21}",
            STYLE_MUTED.apply_to("Operations"),
            VALUE_STYLE.apply_to(format_ui_left_decimal(d.op_count.into(), 0)),
        ),
        format!(
            "  {:<16} avg: {:<21}",
            "",
            VALUE_STYLE.apply_to(format_ui_left_decimal(d.average_operation, 2)),
        ),
        savings_line,
        savings_bar_line,
    ];

    let mut exp_lines: Vec<String> = d
        .top_expenses
        .iter()
        .map(|e| {
            format!(
                "  {} {}  {:>14}  {}",
                STYLE_MUTED.apply_to(format!("{:<7}", format_id_short(&e.op_id))),
                STYLE_MUTED.apply_to(&e.op_date),
                DEBIT_STYLE.apply_to(format!("{:.2}", e.amount).separate_with_commas()),
                truncate_text(&e.description, 22),
            )
        })
        .collect();
    exp_lines.push(String::new());

    let max = fin_lines.len().max(exp_lines.len());
    for i in 0..max {
        let left = fin_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let right = exp_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        println!("{:<45}{}{}", left, sep, right);
    }

    println!();

    // ── Section 2 : Top categories (gauche) + Top counterparties (droite) ──
    let h_left = format!("{:<45}", TITLE_STYLE.apply_to("Top 5 categories"));
    let h_right = format!("{}", TITLE_STYLE.apply_to("Top 5 counterparties"));
    println!("{}{}{}", h_left, sep, h_right);
    println!("{}──┼──{}", "─".repeat(45), "─".repeat(52));

    let cat_lines: Vec<String> = d
        .top_categories
        .iter()
        .map(|c| {
            format!(
                "  {:<18}  {:>14}  {:>6.1}%",
                truncate_text(&c.name, 17),
                DEBIT_STYLE.apply_to(format!("{:.2}", c.total_debit).separate_with_commas()),
                c.debit_percentage,
            )
        })
        .collect();

    let cp_lines: Vec<String> = d
        .top_counterparties
        .iter()
        .map(|c| {
            format!(
                "  {:<18}  {:>14}  {:>6.1}%",
                truncate_text(&c.name, 17),
                DEBIT_STYLE.apply_to(format!("{:.2}", c.total_debit).separate_with_commas()),
                c.debit_percentage,
            )
        })
        .collect();

    let max = cat_lines.len().max(cp_lines.len());
    for i in 0..max {
        let left = cat_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let right = cp_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        println!("{:<45}{}{}", left, sep, right);
    }

    println!();
}
