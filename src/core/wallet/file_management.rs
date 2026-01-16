// src/core/wallet/file_management.rs

use anyhow::{Result, anyhow};
use std::fs::File;
use std::fs;
use std::io;

use std::path::Path;
use zip::write::{FileOptions, ZipWriter};
use zip::ZipArchive;
use walkdir::WalkDir;

use super::operation::Operation;
use super::codexi::Codexi;

use crate::core::helpers::get_data_dir;
use crate::core::helpers::get_snapshot_path;

/// Methods for File Management of codexi
impl Codexi {

    /// Save codexi to file
    pub fn save(&self, dir: &Path) -> Result<()> {
        let file_path = dir.join("codexi.dat");

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let encoded = bincode::serialize(self)?;
        fs::write(&file_path, encoded)?;

        log::debug!("codexi: {:?} saved.", file_path);
        Ok(())
    }
    /// Load codexi from file
    pub fn load(dir: &Path) -> Result<Self> {
        let file_path = dir.join("codexi.dat");

        if !file_path.exists() {
            log::warn!("No codexi file , create a empty file");
            return Ok(Self::default());
        }

        let bytes = fs::read(&file_path)?;
        let codexi = bincode::deserialize(&bytes)?;

        log::debug!("File: {:?} loaded.", file_path);
        Ok(codexi)

    }
    /// Export to toml
    pub fn export_toml(&self, dir: &Path) -> Result<()> {
        let file_path = dir.join("codexi.toml");

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("{}", e))?;


        fs::write(&file_path, toml_str)?;
        log::info!("Export toml saved to {:?}", file_path);
        Ok(())
    }
    /// Import from toml
    pub fn import_toml(dir: &Path) -> Result<Self> {
        let file_path = dir.join("codexi.toml");

        let content = fs::read_to_string(&file_path)?;
        let mut codexi: Codexi = toml::from_str(&content)
            .map_err(|e| anyhow!("{}", e))?;

        codexi.operations.sort_by_key(|o| o.date);
        log::info!("Import toml: {:?} loaded.", file_path);
        Ok(codexi)
    }
    /// Export to csv
    pub fn export_csv(&self, dir: &Path) -> Result<()> {
        let file_path = dir.join("codexi.csv");

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = fs::File::create(&file_path)?;
        let mut wtr = csv::Writer::from_writer(file);

        for op in &self.operations {
            wtr.serialize(op)
                .map_err(|e| anyhow!("{}", e))?;
        }

        wtr.flush()?;
        log::info!("Export csv saved to {:?}", file_path);
        Ok(())
    }
    /// Import from csv
    pub fn import_csv(dir: &Path) -> Result<Self> {
        let file_path = dir.join("codexi.csv");

        let file = fs::File::open(&file_path)?;
        let mut rdr = csv::Reader::from_reader(file);

        let mut operations = Vec::new();

        for result in rdr.deserialize::<Operation>() {
            let op: Operation = result
                .map_err(|e| anyhow!("{}", e))?;
            operations.push(op);
        }
        operations.sort_by_key(|o| o.date);
        log::info!("Import csv: {:?} loaded", file_path);
        Ok(Codexi { operations })
    }
    /// List snapshot files
    pub fn list_snapshot() -> Result<Vec<String>> {

        let data_dir = get_data_dir()?;
        let snapshot_dir = data_dir.join("snapshots");
        let mut files = Vec::new();

        if snapshot_dir.exists() {
            for entry in fs::read_dir(snapshot_dir)? {
                let entry = entry?;
                let file_name = entry.file_name().to_string_lossy().to_string();

                if file_name.starts_with("codexi_") && file_name.ends_with(".snp") {
                    files.push(file_name);
                }
            }
        }
        files.sort();
        Ok(files)
    }
    /// Restore a snapshot file
    /// The filename is just the file name, not the full path
    pub fn restore_snapshot(filename: &str) -> Result<Self> {
        let data_dir = get_data_dir()?;
        let file_path = data_dir.join("snapshots").join(filename);

        let data = fs::read(&file_path)?;
        let codexi: Codexi = bincode::deserialize(&data)
            .map_err(|e| anyhow!("{}", e))?;

        log::info!("Snapshot {} restored", file_path.display());

        Ok(codexi)
    }

    /// Create a snapshot of the current codexi state
    pub fn snapshot(&self) -> Result<()> {

        let file_path = get_snapshot_path()?;
        let data = bincode::serialize(self)
            .map_err(|e| anyhow!("{}", e))?;

        fs::write(&file_path, data)?;

        log::info!("snapshot done to {:?}", file_path);
        Ok(())
    }
    /// Creates a complete ZIP backup of the application's data directory.
    /// The `target_path` is the FULL path where the ZIP file should be written.
    /// It includes all files except internal snapshots.
    pub fn backup(target_path: &Path) -> Result<()> {
        let data_dir = get_data_dir()?;
        let internal_snapshot_dir = data_dir.join("snapshots");

        // The data directory SHALL exist and contain at least the codexi.dat file
        if !data_dir.exists() {
            return Err(anyhow!("The data directory ({}) does not exist.", data_dir.display()));
        }

        // 2. Create the ZIP file
        let file = File::create(target_path)?;
        let mut zip = ZipWriter::new(file);

        // Standard options for compression (Deflated)
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755); // Standard Unix permissions if necessary

        // 3. Iterate the data directory (including codexi.dat and archives/, exclude snapshot)
        for entry in WalkDir::new(&data_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.starts_with(&internal_snapshot_dir) && path != internal_snapshot_dir {
                continue;
            }

            // Paths in the ZIP to be relative to the data_dir, not absolute.
            let name_in_zip = path.strip_prefix(&data_dir)
                .map_err(|_| anyhow!("Failure to calculate relative path for archive."))?
                .to_path_buf();

            if path.is_file() {
                // Add teh ZIP file
                let name_in_zip_str = name_in_zip.to_str().ok_or_else(|| anyhow!("Path invalid (non-UTF8)."))?;

                // Avoid adding temporary or locked files if present (non-standard)
                if name_in_zip_str.contains(".temp") { continue; }

                zip.start_file(name_in_zip_str, options)?;
                io::copy(&mut File::open(path)?, &mut zip)?;

            } else if path.is_dir() && name_in_zip.as_os_str().len() != 0 {
                // Add the directory (only if it is not the root directory itself)
                let name_in_zip_str = name_in_zip.to_str().ok_or_else(|| anyhow!("Path invalid (non-UTF8)."))?;
                zip.add_directory(name_in_zip_str, options)?;
            }
        }

        zip.finish()?;
        log::info!("Full backup successful to: {}", target_path.display());
        Ok(())
    }
    /// Restores the contents of a full ZIP backup to the application's data directory.
    /// The `zip_path` is the FULL path to the backup ZIP file.
    /// Existing files in the data directory will be overwritten.
    pub fn restore(zip_path: &Path) -> Result<()> {

        let data_dir = get_data_dir()?;
        let file = File::open(zip_path)?;

        // Attempting to create the ZIP archive
        let mut archive = ZipArchive::new(file)?;

        log::warn!("Restoration in progress. Existing files in {} will be overwritten.", data_dir.display());

        // Iterate over all files in the archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            // The destination path is data_dir + the path to the file in the ZIP archive
            let outpath = data_dir.join(file.mangled_name());

            if file.is_dir() {
                // Create the directories (e.g., 'archives/')
                fs::create_dir_all(&outpath)?;
            } else if file.is_file() {

                // Ensure that the parent directory exists (in the case of files in 'archives/')
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }

                // Write the contents of the file
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;

                log::debug!("Restore : {}", outpath.file_name().unwrap_or_default().to_string_lossy());
            }
        }

        log::info!("Complete restore successful. The codexi has been reloaded from the backup.");
        Ok(())
    }
    /// List archive files
    /// The archive files are stored in the "archives" subdirectory of the data directory.
    pub fn list_archives() -> Result<Vec<String>> {
        let data_dir = get_data_dir()?;
        let archive_dir = data_dir.join("archives");
        let mut files = Vec::new();

        if archive_dir.exists() {
            for entry in fs::read_dir(archive_dir)? {
                let entry = entry?;
                let file_name = entry.file_name().to_string_lossy().to_string();

                if file_name.starts_with("codexi_") && file_name.ends_with(".cld") {
                    files.push(file_name);
                }
            }
        }
        files.sort();
        Ok(files)
    }
    /// Load an archive file (view only)
    pub fn load_archive(filename: &str) -> Result<Self> {
         let data_dir = get_data_dir()?;
        let file_path = data_dir.join("archives").join(filename);
        let data = fs::read(&file_path)?;
        let codexi: Codexi = bincode::deserialize(&data)
            .map_err(|e| anyhow!("{}", e))?;
        Ok(codexi)
    }

}
