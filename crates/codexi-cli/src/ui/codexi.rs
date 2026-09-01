use codexi::{dto::CodexiSettingsItem, file_management::CodexiInfos};

use crate::ui::{STYLE_MUTED, TITLE_STYLE, VALUE_STYLE, format_bytes, label};

/// view of the codexi infos.
pub fn view_codexi_infos(datas: &CodexiInfos) {
    println!();
    println!("📒 {}", TITLE_STYLE.apply_to("Infos"));
    println!("{}", "─".repeat(80));
    println!("  {} {}", label("Data version", 18), datas.data_version);
    println!("  {} {}", label("Exchange version", 18), datas.exchange_version);
    println!("  {} {}", label("Storage format", 18), datas.storage_format);
    println!("  {} {}", label("data directory", 18), datas.data_dir);

    println!();
    println!("💰 {}", TITLE_STYLE.apply_to("Codexi"));
    println!("{}", "─".repeat(80));
    println!("  {} {}", label("Accounts", 27), datas.codexi_account_count);
    println!(
        "  {} {}",
        label("Operations(incl. archives)", 27),
        datas.codexi_operation_count
    );
    println!("  {} {}", label("Banks", 27), datas.codexi_bank_count);
    println!("  {} {}", label("Currencies", 27), datas.codexi_currency_count);
    println!("  {} {}", label("Categories", 27), datas.codexi_category_count);
    println!("  {} {}", label("Counterparty", 27), datas.codexi_counterparty_count);
    println!();
    let usage = &datas.disk_usage;
    println!("📦 {}", TITLE_STYLE.apply_to("Disk usage"));
    println!("{}", "─".repeat(80));
    println!("  data_dir/");
    println!(
        "    {:<18} {:<10}",
        STYLE_MUTED.apply_to("codexi.dat"),
        VALUE_STYLE.apply_to(format_bytes(usage.data_dir.codexi.size_bytes))
    );
    println!(
        "    {:<18} {:<10} {} files",
        STYLE_MUTED.apply_to("snapshots/"),
        VALUE_STYLE.apply_to(format_bytes(usage.data_dir.snapshots.total_bytes)),
        usage.data_dir.snapshots.file_count
    );
    println!(
        "    {:<18} {:<10} {} account, {} files",
        STYLE_MUTED.apply_to("archives/"),
        VALUE_STYLE.apply_to(format_bytes(usage.data_dir.archives.total_bytes)),
        usage.data_dir.archives.account_count,
        usage.data_dir.archives.file_count
    );
    println!("  {}", "─".repeat(30));
    println!(
        "  {:<20} {:<10}",
        STYLE_MUTED.apply_to("total data_dir"),
        VALUE_STYLE.apply_to(format_bytes(usage.data_dir.total_bytes))
    );

    println!();
    println!(
        "  {:<20} {:<10} {} restore points",
        STYLE_MUTED.apply_to("trash/"),
        VALUE_STYLE.apply_to(format_bytes(usage.trash.total_bytes)),
        usage.trash.restore_point_count
    );

    println!();
    println!("{}", "─".repeat(80));
    println!(
        "  {:<20} {}",
        TITLE_STYLE.apply_to("TOTAL"),
        VALUE_STYLE.apply_to(format_bytes(usage.total_bytes))
    );
    println!();
}

pub fn view_codexi_settings(settings: &CodexiSettingsItem) {
    println!();
    println!("{}", TITLE_STYLE.apply_to("Codexi settings"));
    println!(" Language: {}", settings.language);
    println!(" Currency: {}", settings.default_currency);
    println!();
}
