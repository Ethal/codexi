// src/lib.rs

#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::struct_excessive_bools)]

pub mod core;
pub mod dto;
pub mod exchange;
pub mod file_management;
pub mod logic;
pub mod types;

pub const CODEXI_MAGIC: [u8; 6] = *b"CODEXI";
// V1 to V2 : add FileEnvelope and add next_op_id in codexi structure.
// V2 to V3 : implementation multi account.
pub const CODEXI_DATA_FORMAT_VERSION: u16 = 3;
// V1 to V2 : implementation multi account.
pub const CODEXI_EXCHANGE_FORMAT_VERSION: u16 = 2;
// V1: implementation of the a manifest for the backup.
pub const BACKUP_FORMAT_VERSION: u16 = 1;
