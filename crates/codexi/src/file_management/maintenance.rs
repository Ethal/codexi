// src/file_management/maintenance.rs

use serde::Serialize;
use std::fs;
use walkdir::WalkDir;

use crate::CODEXI_DATA_FORMAT_VERSION;
use crate::CODEXI_EXCHANGE_FORMAT_VERSION;
use crate::core::DataPaths;
use crate::file_management::{FileMaintenanceError, FileManagement, StorageFormat};
use crate::logic::codexi::Codexi;

#[derive(Debug, Clone, Serialize)]
pub struct CodexiInfos {
    pub codexi_account_count: usize,
    pub codexi_operation_count: usize,
    pub codexi_bank_count: usize,
    pub codexi_currency_count: usize,
    pub codexi_category_count: usize,
    pub codexi_counterparty_count: usize,
    pub data_version: String,
    pub exchange_version: String,
    pub storage_format: StorageFormat,
    pub disk_usage: DiskUsage,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskUsage {
    pub data_dir: DataDirUsage,
    pub trash: TrashUsage,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataDirUsage {
    pub codexi: CodexiFileUsage,
    pub snapshots: SnapshotsUsage,
    pub archives: ArchivesUsage,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodexiFileUsage {
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotsUsage {
    pub file_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchivesUsage {
    pub account_count: usize,
    pub file_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrashUsage {
    pub restore_point_count: usize,
    pub total_bytes: u64,
}

impl FileManagement {
    /// Clear all data file related to codexi
    pub fn clear_data(paths: &DataPaths) -> Result<(), FileMaintenanceError> {
        let codexi = &paths.main_file;
        let snapshots = &paths.snapshots_dir;
        let archives = &paths.archives_dir;

        // CASE 1 : no codexi.dat → no valid state → cleaning
        if !codexi.exists() {
            if snapshots.exists() {
                fs::remove_dir_all(snapshots)?;
            }
            if archives.exists() {
                fs::remove_dir_all(archives)?;
            }
            return Ok(());
        }

        // CASE 2 : valid state → move to codexi trash
        let trash_path = paths.trash_path(); // include data
        fs::create_dir_all(&trash_path)?;

        fs::rename(
            codexi,
            trash_path.join(format!("{}.{}", DataPaths::APP_NAME, DataPaths::EXT_DATA)),
        )?;

        if snapshots.exists() {
            fs::rename(snapshots, trash_path.join(DataPaths::DIR_SNAPSHOTS))?;
        }

        if archives.exists() {
            fs::rename(archives, trash_path.join(DataPaths::DIR_ARCHIVES))?;
        }

        Ok(())
    }

    pub fn restore_trash(paths: &DataPaths, date: String) -> Result<(), FileMaintenanceError> {
        let trash_snapshot = paths.trash_dir.join(date);

        if !trash_snapshot.exists() {
            return Err(FileMaintenanceError::NoTrashSnapshot(format!(
                "trash snapshot not found: {:?}",
                trash_snapshot
            )));
        }

        let main_file = format!("{}.{}", DataPaths::APP_NAME, DataPaths::EXT_DATA);
        let src_codexi = trash_snapshot.join(&main_file);
        if !src_codexi.exists() {
            return Err(FileMaintenanceError::InvalidTrashSnapshot(format!(
                "invalid snapshot (missing {}): {}",
                main_file,
                trash_snapshot.display()
            )));
        }

        let codexi = &paths.main_file;

        if codexi.exists() {
            Self::clear_data(paths)?;
        }

        // cleaning
        let snapshots = &paths.snapshots_dir;
        if snapshots.exists() {
            fs::remove_dir_all(snapshots)?;
        }

        let archives = &paths.archives_dir;
        if archives.exists() {
            fs::remove_dir_all(archives)?;
        }

        // Restore
        fs::rename(src_codexi, &paths.main_file)?;

        let src_snapshots = trash_snapshot.join(DataPaths::DIR_SNAPSHOTS);
        if src_snapshots.exists() {
            fs::rename(src_snapshots, &paths.snapshots_dir)?;
        }

        let src_archives = trash_snapshot.join(DataPaths::DIR_ARCHIVES);
        if src_archives.exists() {
            fs::rename(src_archives, &paths.archives_dir)?;
        }

        fs::remove_dir_all(&trash_snapshot)?;

        Ok(())
    }

    pub fn clean_trash(paths: &DataPaths) -> Result<(), FileMaintenanceError> {
        let trash = &paths.trash_dir;

        if trash.exists() {
            fs::remove_dir_all(trash)?;
        }

        Ok(())
    }
    /// Get Codexi infos
    pub fn codexi_infos(paths: &DataPaths, data: &Codexi) -> Result<CodexiInfos, FileMaintenanceError> {
        let codexi_account_count = data.accounts.len();
        let codexi_bank_count = data.banks.count();
        let codexi_currency_count = data.currencies.count();
        let codexi_category_count = data.categories.count();
        let codexi_counterparty_count = data.counterparties.count();

        let mut codexi_operation_count = 0;
        for acc in data.accounts.iter() {
            codexi_operation_count += acc.operations.len();
            for chk in acc.checkpoints.iter() {
                codexi_operation_count += chk.archive_operation_count;
            }
        }

        let codexi_path = &paths.main_file;
        let codexi_size = if codexi_path.exists() {
            fs::metadata(codexi_path)?.len()
        } else {
            0
        };

        let codexi = CodexiFileUsage {
            size_bytes: codexi_size,
        };

        // snapshots/
        let snapshots_dir = &paths.snapshots_dir;
        let mut snapshots_file_count = 0usize;
        let mut snapshots_total_bytes = 0u64;

        if snapshots_dir.exists() {
            for entry in WalkDir::new(snapshots_dir).min_depth(1).max_depth(1) {
                let entry = entry?;
                let path = entry.path();
                if path.is_file()
                    && let Some(ext) = path.extension()
                    && ext == "snp"
                {
                    snapshots_file_count += 1;
                    snapshots_total_bytes += entry.metadata()?.len();
                }
            }
        }

        let snapshots = SnapshotsUsage {
            file_count: snapshots_file_count,
            total_bytes: snapshots_total_bytes,
        };

        // archives/
        let archives_dir = &paths.archives_dir;
        let mut account_count = 0usize;
        let mut archives_file_count = 0usize;
        let mut archives_total_bytes = 0u64;

        if archives_dir.exists() {
            for entry in WalkDir::new(archives_dir).min_depth(1) {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() && entry.depth() == 1 {
                    account_count += 1;
                }

                if path.is_file()
                    && let Some(ext) = path.extension()
                    && ext == "cld"
                {
                    archives_file_count += 1;
                    archives_total_bytes += entry.metadata()?.len();
                }
            }
        }

        let archives = ArchivesUsage {
            account_count,
            file_count: archives_file_count,
            total_bytes: archives_total_bytes,
        };

        // total data_dir (without trash)
        let data_dir_total_bytes = codexi_size + snapshots_total_bytes + archives_total_bytes;

        let data_dir_usage = DataDirUsage {
            codexi,
            snapshots,
            archives,
            total_bytes: data_dir_total_bytes,
        };

        // trash/
        let trash_dir = &paths.trash_dir;
        let mut restore_point_count = 0usize;
        let mut trash_total_bytes = 0u64;

        if trash_dir.exists() {
            for entry in WalkDir::new(trash_dir).min_depth(1) {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() && entry.depth() == 1 {
                    restore_point_count += 1;
                }

                if path.is_file() {
                    trash_total_bytes += entry.metadata()?.len();
                }
            }
        }

        let trash = TrashUsage {
            restore_point_count,
            total_bytes: trash_total_bytes,
        };

        let total_bytes = data_dir_total_bytes + trash_total_bytes;

        let disk_usage = DiskUsage {
            data_dir: data_dir_usage,
            trash,
            total_bytes,
        };

        let result = CodexiInfos {
            codexi_account_count,
            codexi_operation_count,
            codexi_bank_count,
            codexi_currency_count,
            codexi_category_count,
            codexi_counterparty_count,
            data_version: CODEXI_DATA_FORMAT_VERSION.to_string(),
            exchange_version: CODEXI_EXCHANGE_FORMAT_VERSION.to_string(),
            storage_format: StorageFormat::Ciborium,
            disk_usage,
            data_dir: paths.data_root.to_string_lossy().to_string(),
        };

        Ok(result)
    }
}
