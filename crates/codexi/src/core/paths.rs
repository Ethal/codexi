// src/core/paths.rs

use chrono::{Local, NaiveDate};
use nulid::Nulid;
use std::path::{Path, PathBuf};

use crate::core::{format_date, format_date_time_short, format_id};

/*
// Disk structure of Codexi (data dirctory fr.ethal.codexi)
├── <APP_NAME>.<EXT_MAIN>
├── <DIR_ARCHIVES>
│   └── <ACCOUNT_ID>
│       ├── <ACCOUNT_ID>_<APP_NAME>_<YYYY>-<MM>-<DD>.<EXT_ARCHIVE>
│       ├── ...
│       └── ...
├── <DIR_SNAPSHOTS>
│   └── <APP_NAME>_<YYYYMMDD>_<HHMMSS>.<EXT_SNAPSHOT>
│   └── ....
├── <DIR_TMP>
│   └── ......
└── <DIR_TRASH>
    └── <YYYYMMDD>_<HHMMSS>
        ├── archives/...
        ├── codexi.dat
        └── snapshots/...

// Disk structure of Codexi (config dirctory fr.ethal.codexi)
└── <APP_NAME>.<EXT_CFG>
*/

/// A resolved file path with its filename
pub struct ResolvedPath {
    pub path: PathBuf,
    pub filename: String,
}

pub struct DataPaths {
    pub data_root: PathBuf,     // data_dir
    pub config_root: PathBuf,   // config_dir
    pub main_file: PathBuf,     // data_dir/codexi.dat
    pub config_file: PathBuf,   // config_dir/codexi.cfg
    pub archives_dir: PathBuf,  // data_dir/archives/
    pub snapshots_dir: PathBuf, // data_dir/snapshots/
    pub tmp_dir: PathBuf,       // data_dir/tmp/
    pub trash_dir: PathBuf,     // data_dir/trash/
}

impl DataPaths {
    pub(crate) const APP_NAME: &'static str = "codexi";
    pub(crate) const EXT_DATA: &'static str = "dat";
    pub(crate) const EXT_CFG: &'static str = "cfg";
    pub(crate) const EXT_ARCHIVE: &'static str = "cld";
    pub(crate) const EXT_SNAPSHOT: &'static str = "snp";
    pub(crate) const DIR_ARCHIVES: &'static str = "archives";
    pub(crate) const DIR_SNAPSHOTS: &'static str = "snapshots";
    pub(crate) const DIR_TMP: &'static str = "tmp";
    pub(crate) const DIR_TRASH: &'static str = "trash";

    pub fn new(data_dir: &Path, config_dir: &Path) -> Self {
        let data_root = data_dir.to_path_buf();
        let config_root = config_dir.to_path_buf();

        Self {
            main_file: data_root.join(format!("{}.{}", Self::APP_NAME, Self::EXT_DATA)),
            config_file: config_root.join(format!("{}.{}", Self::APP_NAME, Self::EXT_CFG)),

            archives_dir: data_root.join(Self::DIR_ARCHIVES),
            snapshots_dir: data_root.join(Self::DIR_SNAPSHOTS),
            tmp_dir: data_root.join(Self::DIR_TMP),
            trash_dir: data_root.join(Self::DIR_TRASH),

            data_root,
            config_root,
        }
    }

    /// archives/<account_id>/
    pub fn archive_dir(&self, account_id: &Nulid) -> PathBuf {
        self.archives_dir.join(format_id(*account_id))
    }

    /// archives/<account_id>/<account_id>_codexi_<date>.cld
    pub fn archive_path(&self, account_id: &Nulid, date: &NaiveDate) -> ResolvedPath {
        let filename = format!(
            "{}_{}_{}.{}",
            format_id(*account_id),
            Self::APP_NAME,
            format_date(*date),
            Self::EXT_ARCHIVE,
        );
        let path = self.archive_dir(account_id).join(&filename);
        ResolvedPath { path, filename }
    }

    /// snapshots/codexi_<timestamp>.snp
    pub fn snapshot_path(&self) -> ResolvedPath {
        let filename = format!(
            "{}_{}.{}",
            Self::APP_NAME,
            format_date_time_short(Local::now().naive_local()),
            Self::EXT_SNAPSHOT,
        );
        let path = self.snapshots_dir.join(&filename);

        ResolvedPath { path, filename }
    }

    /// trash/<timestamp>/
    pub fn trash_path(&self) -> PathBuf {
        self.trash_dir.join(format_date_time_short(Local::now().naive_local()))
    }
}
