use std::collections::BTreeMap;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub const STATE_VERSION: &str = "2.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDefaults {
    pub default_branch: String,
    pub shallow_clone: bool,
    pub shallow_depth: u32,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            default_branch: String::from("main"),
            shallow_clone: true,
            shallow_depth: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRecord {
    pub name: String,
    pub url: String,
    pub path: Utf8PathBuf,
    pub added_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub commit_id: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub version: String,
    pub defaults: ConfigDefaults,
    pub repos: BTreeMap<String, RepoRecord>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            version: String::from(STATE_VERSION),
            defaults: ConfigDefaults::default(),
            repos: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadingEntry {
    pub id: String,
    pub repo_name: String,
    pub repo_url: String,
    pub commit_id: String,
    pub branch: Option<String>,
    pub subdir: Option<String>,
    pub target_path: String,
    pub loaded_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
    pub working_directory: Utf8PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadingFile {
    pub version: String,
    pub entries: Vec<LoadingEntry>,
}

impl Default for LoadingFile {
    fn default() -> Self {
        Self {
            version: String::from(STATE_VERSION),
            entries: Vec::new(),
        }
    }
}
