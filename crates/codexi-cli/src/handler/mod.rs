// src/handler/mod.rs

mod account;
mod admin;
mod bank;
mod category;
mod counterparty;
mod currency;
mod data;
mod history;
mod loan;
mod operation;
mod report;
mod root;
mod settings;

pub use root::handle_root_command;
