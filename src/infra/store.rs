use camino::Utf8Path;
use uuid::Uuid;

use crate::domain::error::{GrfError, Result};
use crate::domain::model::{ConfigFile, LoadingEntry, LoadingFile, RepoRecord};
use crate::infra::{fs, paths::GrfPaths};

pub struct StateStore {
    paths: GrfPaths,
}

impl StateStore {
    pub fn new(paths: GrfPaths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &GrfPaths {
        &self.paths
    }

    pub fn load_config(&self) -> Result<ConfigFile> {
        fs::read_json_or_default(&self.paths.config_path)
    }

    pub fn save_config(&self, config: &ConfigFile) -> Result<()> {
        fs::write_json_atomic(&self.paths.config_path, config)
    }

    pub fn load_loading(&self) -> Result<LoadingFile> {
        fs::read_json_or_default(&self.paths.loading_path)
    }

    pub fn save_loading(&self, loading: &LoadingFile) -> Result<()> {
        fs::write_json_atomic(&self.paths.loading_path, loading)
    }

    pub fn ensure_layout(&self) -> Result<()> {
        self.paths.ensure_root()
    }

    pub fn resolve_repo<'a>(&self, config: &'a ConfigFile, needle: &str) -> Result<&'a RepoRecord> {
        if let Some(repo) = config.repos.get(needle) {
            return Ok(repo);
        }

        let mut matches = config
            .repos
            .values()
            .filter(|repo| {
                repo.name == needle
                    || repo.name.ends_with(&format!("/{needle}"))
                    || repo
                        .name
                        .rsplit('/')
                        .next()
                        .is_some_and(|part| part == needle)
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => Err(GrfError::RepoNotFound {
                name: needle.to_string(),
            }),
            1 => Ok(matches.remove(0)),
            _ => Err(GrfError::AmbiguousRepoName {
                name: needle.to_string(),
                matches: matches.iter().map(|repo| repo.name.clone()).collect(),
            }),
        }
    }

    pub fn upsert_loading_entry(
        &self,
        loading: &mut LoadingFile,
        mut entry: LoadingEntry,
    ) -> LoadingEntry {
        if let Some(existing) = loading.entries.iter_mut().find(|candidate| {
            candidate.working_directory == entry.working_directory
                && candidate.target_path == entry.target_path
        }) {
            existing.repo_name = entry.repo_name.clone();
            existing.repo_url = entry.repo_url.clone();
            existing.commit_id = entry.commit_id.clone();
            existing.branch = entry.branch.clone();
            existing.subdir = entry.subdir.clone();
            existing.updated_at = entry.updated_at.take();
            return existing.clone();
        }

        if entry.id.is_empty() {
            entry.id = Uuid::new_v4().to_string();
        }
        loading.entries.push(entry.clone());
        entry
    }

    pub fn loading_entries_for_workspace<'a>(
        &self,
        loading: &'a LoadingFile,
        cwd: &Utf8Path,
    ) -> Vec<&'a LoadingEntry> {
        loading
            .entries
            .iter()
            .filter(|entry| entry.working_directory == cwd)
            .collect()
    }

    pub fn remove_loading_entry(
        &self,
        loading: &mut LoadingFile,
        workspace: &Utf8Path,
        target_path: &str,
    ) -> bool {
        let before = loading.entries.len();
        loading.entries.retain(|entry| {
            !(entry.working_directory == workspace && entry.target_path == target_path)
        });
        before != loading.entries.len()
    }
}
