// src/core/mod.rs

mod error;
mod format;
mod fs;
mod parse;
mod paths;
pub mod serde_nulid;
mod validation;
mod warning;

pub const ID_MIN_SHORT_LEN: usize = 5;

pub use error::CoreError;
pub use format::{
    default_zero, format_date, format_date_time_long, format_date_time_short, format_decimal, format_id,
    format_id_short, format_optional, format_optional_date, format_optional_id, format_optional_id_short,
    format_optional_path, format_optional_text, format_optional_u32, format_path, format_text, format_time, yes_no,
};
pub use fs::{get_config_dir, get_data_dir, get_documents_dir};
pub use parse::{
    normalize_string, normalize_vec_input, parse_date, parse_decimal, parse_id, parse_optional, parse_optional_date,
    parse_optional_decimal, parse_optional_id, parse_optional_path, parse_optional_u32, parse_path, parse_text,
    parse_u32, resolve_or_generate_id,
};
pub use paths::DataPaths;
pub use validation::validate_text_rules;
pub use warning::{CoreWarning, CoreWarningKind};
