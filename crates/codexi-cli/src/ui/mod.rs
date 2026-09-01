// src/ui/mod.rs

mod account;
mod balance;
mod bank;
mod category;
mod codexi;
mod counterparty;
mod currency;
mod display;
mod helpers;
mod loan;
mod operation;
mod rate;
mod report;
mod tree;

pub use account::*;
pub use balance::*;
pub use bank::view_bank;
pub use category::*;
pub use codexi::*;
pub use counterparty::*;
pub use currency::view_currency;
pub use display::*;
pub use helpers::*;
pub use loan::*;
pub use operation::*;
pub use rate::*;
pub use report::*;
pub use tree::*;

use console::Style;
const TITLE_STYLE: Style = Style::new().cyan().bold();
const NOTE_STYLE: Style = Style::new().cyan().italic();
const STYLE_MUTED: Style = Style::new().dim();
const STYLE_NORMAL: Style = Style::new();
const STYLE_HIGHLIGHT: Style = Style::new().yellow();
const STYLE_DANGER: Style = Style::new().red();
const STYLE_CAUTION: Style = Style::new().magenta().bold();
const DEBIT_STYLE: Style = Style::new().red();
const CREDIT_STYLE: Style = Style::new().green();
const VALUE_STYLE: Style = Style::new().yellow().bold();
