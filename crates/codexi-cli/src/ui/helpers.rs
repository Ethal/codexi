use console::style;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use thousands::Separable;

use codexi::dto::{BankItem, CurrencyItem};

use crate::ui::STYLE_MUTED;

/// Truncate text for ui
pub fn truncate_text(desc: &str, max_width: usize) -> String {
    // If the visible length is already OK → simple formatting
    if desc.chars().count() <= max_width {
        return format!("{:<width$}", desc, width = max_width);
    }

    // Otherwise → truncate without ever breaking a UTF-8 character
    let visible = max_width.saturating_sub(3);

    let truncated: String = desc.chars().take(visible).collect();

    format!("{:<width$}", format!("{}...", truncated), width = max_width)
}

pub fn label(text: &str, width: usize) -> impl std::fmt::Display {
    STYLE_MUTED.apply_to(format!("{:<width$}", text, width = width))
}

/// Utility function for the visual toolbar — centered on 0
/// [-100%  ░░░░████|░░░░░░░░  +100%]
pub fn draw_savings_bar(rate: Decimal, width: usize) -> String {
    let half = width / 2;
    let normalized = rate.abs().min(Decimal::ONE_HUNDRED) / Decimal::ONE_HUNDRED;
    let filled = (normalized * Decimal::from(half)).to_usize().unwrap_or(0);
    let empty = half - filled;

    if rate <= Decimal::ZERO {
        // negative: fill grows left from center
        format!(
            "{}{}|{}",
            style("░".repeat(empty)).dim(),
            style("█".repeat(filled)).red(),
            style("░".repeat(half)).dim(),
        )
    } else {
        // positive: fill grows right from center
        format!(
            "{}|{}{}",
            style("░".repeat(half)).dim(),
            style("█".repeat(filled)).green(),
            style("░".repeat(empty)).dim(),
        )
    }
}

pub fn format_optional_currency_item(currency: &Option<CurrencyItem>) -> String {
    match currency {
        Some(c) => c.code.to_string(),
        None => "─".to_string(),
    }
}

pub fn format_optional_bank_item(bank: &Option<BankItem>) -> String {
    match bank {
        Some(b) => b.name.to_string(),
        None => "─".to_string(),
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;

    if b < KB {
        format!("{} B", bytes)
    } else if b < MB {
        format!("{:.2} KB", b / KB)
    } else if b < GB {
        format!("{:.2} MB", b / MB)
    } else {
        format!("{:.2} GB", b / GB)
    }
}

pub fn format_ui_left_decimal(value: Decimal, dec_place: usize) -> String {
    format!("{:.dec_place$}", value).separate_with_commas()
}
