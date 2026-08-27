// src/ui/operations.rs

use rust_decimal::Decimal;
use thousands::Separable;

use codexi::{
    core::{format_id_short, format_optional_id_short, format_optional_text},
    dto::SearchOperationItem,
};

use crate::ui::{CREDIT_STYLE, DEBIT_STYLE, NOTE_STYLE, TITLE_STYLE, VALUE_STYLE, label};

/// Display a detailed view of a single operation
pub fn view_operation(item: &SearchOperationItem) {
    let l = |text| label(text, 14);

    // ── Operation ─────────────────────────────────────────────
    println!();
    println!("{}", TITLE_STYLE.apply_to("Operation"));
    println!("{}", "─".repeat(45));

    println!("  {} {}", l("Id"), VALUE_STYLE.apply_to(&item.id));
    println!("  {} {}", l("Short"), VALUE_STYLE.apply_to(format_id_short(&item.id)));
    println!("  {} {}", l("Date"), item.date);
    println!("  {} {}", l("Type"), item.kind);

    let flow_display = if item.flow == "Debit" {
        DEBIT_STYLE.apply_to(&item.flow)
    } else {
        CREDIT_STYLE.apply_to(&item.flow)
    };
    let can_void_indicator = if item.can_be_void {
        format!("  {}", NOTE_STYLE.apply_to("[can void]"))
    } else {
        String::new()
    };
    println!("  {} {}{}", l("Flow"), flow_display, can_void_indicator);

    let amount_str = format!("{:.2}", item.amount).separate_with_commas();
    let amount_display = if item.flow == "Debit" {
        DEBIT_STYLE.apply_to(&amount_str)
    } else {
        CREDIT_STYLE.apply_to(&amount_str)
    };
    println!("  {} {}", l("Amount"), amount_display);

    let balance_str = format!("{:.2}", item.balance).separate_with_commas();
    let balance_display = if item.balance < Decimal::ZERO {
        DEBIT_STYLE.apply_to(&balance_str)
    } else {
        CREDIT_STYLE.apply_to(&balance_str)
    };
    println!("  {} {}", l("Balance"), balance_display);
    println!("  {} {}", l("Description"), item.description);

    // ── Links ─────────────────────────────────────────────────
    println!();
    println!("{}", TITLE_STYLE.apply_to("Links"));
    println!("{}", "─".repeat(45));

    println!(
        "  {} {}",
        l("Void of"),
        format_optional_id_short(item.void_of.as_deref())
    );
    println!(
        "  {} {}",
        l("Void by"),
        format_optional_id_short(item.void_by.as_deref())
    );

    let transfer_display = match (&item.transfer_id, &item.transfer_account_id) {
        (Some(tid), Some(aid)) => format!("{} (account {})", format_id_short(tid), format_id_short(aid)),
        (Some(tid), None) => format_id_short(tid),
        _ => "—".into(),
    };
    println!("  {} {}", l("Transfer"), transfer_display);

    // ── Context ───────────────────────────────────────────────
    println!();
    println!("{}", TITLE_STYLE.apply_to("Context"));
    println!("{}", "─".repeat(45));

    println!("  {} {}", l("Currency"), format_optional_text(item.currency.as_deref()));
    println!(
        "  {} {}",
        l("Rate"),
        format!("{:.8}", item.exchange_rate).separate_with_commas()
    );
    println!(
        "  {} {}",
        l("Counterparty"),
        format_optional_text(item.counterparty.as_deref())
    );
    println!("  {} {}", l("Category"), format_optional_text(item.category.as_deref()));
    println!("  {} {}", l("Payee"), format_optional_text(item.payee.as_deref()));
    println!(
        "  {} {}",
        l("Reconciled"),
        format_optional_text(item.reconciled.as_deref())
    );

    // ── Meta ──────────────────────────────────────────────────
    println!();
    println!("{}", TITLE_STYLE.apply_to("Meta"));
    println!("{}", "─".repeat(45));

    println!("  {} {}", l("Tags"), format_optional_text(item.tags.as_deref()));
    println!(
        "  {} {}",
        NOTE_STYLE.apply_to(format!("{:<14}", "Note")),
        NOTE_STYLE.apply_to(format_optional_text(item.note.as_deref()))
    );
    println!(
        "  {} {}",
        l("Attachment"),
        format_optional_text(item.attachment.as_deref())
    );

    println!();
}

/// Display raw debug output — for diagnostic purposes
pub fn view_operation_raw(item: &SearchOperationItem) {
    println!("{:#?}", item);
}
