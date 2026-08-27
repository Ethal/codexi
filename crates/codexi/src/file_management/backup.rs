// src/file_management/backup.rs

use chrono::Local;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use walkdir::WalkDir;

use crate::core::{DataPaths, get_documents_dir};
use crate::file_management::{FileBackupError, FileManagement};
use crate::{BACKUP_FORMAT_VERSION, CODEXI_DATA_FORMAT_VERSION};

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    format_version: u16,
    application: String,
    created_at: String,
}

impl Manifest {
    pub fn new(format_version: u16, application: &str, created_at: String) -> Self {
        Self {
            format_version,
            application: application.to_string(),
            created_at,
        }
    }
}

impl FileManagement {
    /// Creates a complete TAR.GZ backup of the application's data and config.
    ///
    /// The backup contains:
    /// - manifest.toml
    /// - data/      application data
    /// - config/    application configuration
    ///
    /// The data directory excludes:
    /// - snapshots/
    /// - tmp/
    /// - trash/
    ///
    /// The returned path is the FULL path to the created archive.
    pub fn create_backup(paths: &DataPaths, target_dir_arg: Option<&str>) -> Result<PathBuf, FileBackupError> {
        let target_path = get_final_backup_path(target_dir_arg)?;

        // The main data file SHALL exist.
        if !paths.main_file.exists() {
            return Err(FileBackupError::NoDirOrFile(format!(
                "The data directory ({:?}) does not exist or contains no main file.",
                paths.data_root
            )));
        }

        let file = File::create(&target_path)?;

        let encoder = GzEncoder::new(file, Compression::default());
        let mut tar = Builder::new(encoder);

        // ---------------------------------------------------------------------
        // Manifest
        // ---------------------------------------------------------------------
        let created_at = Local::now().to_rfc3339();
        let manifest = Manifest::new(BACKUP_FORMAT_VERSION, DataPaths::APP_NAME, created_at);
        let content = toml::to_string_pretty(&manifest)?;

        let mut header = tar::Header::new_gnu();
        header.set_path("manifest.toml")?;
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        tar.append(&header, content.as_bytes())?;

        // ---------------------------------------------------------------------
        // Data
        // ---------------------------------------------------------------------

        for entry in WalkDir::new(&paths.data_root).into_iter().filter_map(Result::ok) {
            let path = entry.path();

            // Exclude internal directories that must never be backed up.
            if path.starts_with(&paths.snapshots_dir)
                || path.starts_with(&paths.tmp_dir)
                || path.starts_with(&paths.trash_dir)
            {
                continue;
            }

            let relative_path = path.strip_prefix(&paths.data_root).map_err(|_| {
                FileBackupError::RelativePath("Failure to calculate relative path for archive.".to_string())
            })?;

            // Skip the root directory itself.
            if relative_path.as_os_str().is_empty() {
                continue;
            }

            // Avoid temporary files.
            if relative_path.to_string_lossy().contains(".temp") {
                continue;
            }

            // If config_dir == data_dir, config_file would otherwise be stored
            // twice: once under data/ and once under config/.
            if path == paths.config_file {
                continue;
            }

            let archive_path = Path::new("data").join(relative_path);

            if path.is_file() {
                tar.append_path_with_name(path, &archive_path)?;
            } else if path.is_dir() {
                tar.append_dir(&archive_path, path)?;
            }
        }

        // ---------------------------------------------------------------------
        // Config
        // ---------------------------------------------------------------------

        if paths.config_file.exists() {
            let archive_path = Path::new("config").join(paths.config_file.file_name().ok_or_else(|| {
                FileBackupError::RelativePath("Failure to determine configuration filename.".to_string())
            })?);

            tar.append_path_with_name(&paths.config_file, archive_path)?;
        }

        // ---------------------------------------------------------------------
        // Finish TAR.GZ
        // ---------------------------------------------------------------------

        let encoder = tar.into_inner()?;
        encoder.finish()?;

        Ok(target_path)
    }

    /// Restores the contents of a TAR.GZ backup into the application's
    /// data and config directories.
    ///
    /// Existing application data is removed before restoration.
    ///
    /// The archive is expected to contain:
    /// - manifest.toml
    /// - data/
    /// - config/
    pub fn restore_backup(paths: &DataPaths, backup_path: &Path) -> Result<(), FileBackupError> {
        // ---------------------------------------------------------------------
        // Clear existing data
        // ---------------------------------------------------------------------

        let codexi = &paths.main_file;

        // If an active ledger exists, use the normal cleanup process.
        // Otherwise remove leftover internal directories manually.
        if codexi.exists() {
            Self::clear_data(paths)?;
        } else {
            if paths.snapshots_dir.exists() {
                fs::remove_dir_all(&paths.snapshots_dir)?;
            }

            if paths.archives_dir.exists() {
                fs::remove_dir_all(&paths.archives_dir)?;
            }
        }

        fs::create_dir_all(&paths.data_root)?;
        fs::create_dir_all(&paths.config_root)?;

        // ---------------------------------------------------------------------
        // Open TAR.GZ
        // ---------------------------------------------------------------------

        let file = File::open(backup_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let mut manifest_found = false;

        // ---------------------------------------------------------------------
        // Extract entries
        // ---------------------------------------------------------------------

        for entry in archive.entries()? {
            let mut entry = entry?;

            let entry_path = entry.path()?.into_owned();

            // Reject absolute paths and parent-directory components.
            //
            // This prevents an archive from extracting outside the intended
            // application directories.
            if entry_path.is_absolute()
                || entry_path
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
            {
                return Err(FileBackupError::RelativePath(format!(
                    "Unsafe path in backup archive: {:?}",
                    entry_path
                )));
            }

            // -------------------------------------------------------------
            // Manifest
            // -------------------------------------------------------------

            if entry_path == Path::new("manifest.toml") {
                let mut content = String::new();
                io::Read::read_to_string(&mut entry, &mut content)?;

                let manifest: Manifest = toml::from_str(&content)?;

                if manifest.format_version != BACKUP_FORMAT_VERSION {
                    return Err(FileBackupError::InvalidData("Incompatible backup version".to_string()));
                }

                if manifest.application != DataPaths::APP_NAME {
                    return Err(FileBackupError::InvalidData(
                        "Incompatible backup application".to_string(),
                    ));
                }

                manifest_found = true;
                continue;
            }
            // -------------------------------------------------------------
            // Data
            // -------------------------------------------------------------

            if let Ok(relative_path) = entry_path.strip_prefix("data") {
                // `data/` itself.
                if relative_path.as_os_str().is_empty() {
                    continue;
                }

                let destination = paths.data_root.join(relative_path);

                if entry.header().entry_type().is_dir() {
                    fs::create_dir_all(&destination)?;
                } else if entry.header().entry_type().is_file() {
                    if let Some(parent) = destination.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    entry.unpack(&destination)?;
                } else {
                    return Err(FileBackupError::RelativePath(format!(
                        "Unsupported TAR entry in data/: {:?}",
                        entry_path
                    )));
                }

                continue;
            }

            // -------------------------------------------------------------
            // Config
            // -------------------------------------------------------------

            if let Ok(relative_path) = entry_path.strip_prefix("config") {
                if relative_path.as_os_str().is_empty() {
                    continue;
                }

                let destination = paths.config_root.join(relative_path);

                if entry.header().entry_type().is_dir() {
                    fs::create_dir_all(&destination)?;
                } else if entry.header().entry_type().is_file() {
                    if let Some(parent) = destination.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    entry.unpack(&destination)?;
                } else {
                    return Err(FileBackupError::RelativePath(format!(
                        "Unsupported TAR entry in config/: {:?}",
                        entry_path
                    )));
                }

                continue;
            }

            // Anything else in the archive is rejected.
            return Err(FileBackupError::RelativePath(format!(
                "Unexpected entry in backup archive: {:?}",
                entry_path
            )));
        }

        // ---------------------------------------------------------------------
        // Manifest is mandatory
        // ---------------------------------------------------------------------

        if !manifest_found {
            return Err(FileBackupError::InvalidData("Backup manifest is missing".to_string()));
        }

        Ok(())
    }
}

/// Determines the full path to the backup archive.
///
/// If `target_dir_arg` is:
/// - a directory -> a default filename is generated
/// - a .tar.gz file -> that filename is used
/// - None -> the user's Documents directory is used
fn get_final_backup_path(target_dir_arg: Option<&str>) -> Result<PathBuf, FileBackupError> {
    let now = Local::now();

    let default_filename = format!(
        "codexi_backup_{}_v{}.tar.gz",
        now.format("%Y%m%d_%H%M%S"),
        CODEXI_DATA_FORMAT_VERSION
    );

    let (target_dir, final_filename) = if let Some(path_str) = target_dir_arg {
        let path = PathBuf::from(path_str);

        let is_backup_file = path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().to_ascii_lowercase().ends_with(".tar.gz"));

        if is_backup_file {
            let filename = path
                .file_name()
                .ok_or_else(|| {
                    FileBackupError::InvalidBackupPath("The path specified for the backup is invalid.".to_string())
                })?
                .to_string_lossy()
                .into_owned();

            let dir = path
                .parent()
                .map(|p| {
                    if p.as_os_str().is_empty() {
                        PathBuf::from(".")
                    } else {
                        p.to_path_buf()
                    }
                })
                .unwrap_or_else(|| PathBuf::from("."));

            (dir, filename)
        } else {
            (path, default_filename)
        }
    } else {
        let documents_dir = get_documents_dir()?;
        (documents_dir, default_filename)
    };

    fs::create_dir_all(&target_dir)?;

    Ok(target_dir.join(final_filename))
}
