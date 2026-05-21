use camino::{Utf8Path, Utf8PathBuf};
use directories::BaseDirs;

use crate::domain::error::{GrfError, Result};

#[derive(Debug, Clone)]
pub struct GrfPaths {
    pub root: Utf8PathBuf,
    pub repos_root: Utf8PathBuf,
    pub config_path: Utf8PathBuf,
    pub loading_path: Utf8PathBuf,
}

impl GrfPaths {
    pub fn discover() -> Result<Self> {
        if let Some(root_override) = std::env::var_os("GRF_ROOT_DIR") {
            let root = to_utf8_path_buf(root_override.into())?;
            return Ok(Self {
                repos_root: root.join("repos"),
                config_path: root.join("config.v2.json"),
                loading_path: root.join("loading.v2.json"),
                root,
            });
        }

        let base_dirs = BaseDirs::new().ok_or(GrfError::HomeDirUnavailable)?;
        let home = to_utf8_path_buf(base_dirs.home_dir().to_path_buf())?;
        let root = home.join(".gitreference");

        Ok(Self {
            repos_root: root.join("repos"),
            config_path: root.join("config.v2.json"),
            loading_path: root.join("loading.v2.json"),
            root,
        })
    }

    pub fn ensure_root(&self) -> Result<()> {
        fs_err::create_dir_all(&self.repos_root)
            .map_err(|source| GrfError::io_write(self.repos_root.to_string(), source))
    }

    pub fn workspace_root(cwd: &Utf8Path) -> Utf8PathBuf {
        cwd.join(".gitreference")
    }
}

pub fn current_dir_utf8() -> Result<Utf8PathBuf> {
    let cwd = std::env::current_dir().map_err(|source| GrfError::io_read(".", source))?;
    to_utf8_path_buf(cwd)
}

pub fn to_utf8_path_buf(path: std::path::PathBuf) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path).map_err(|pathbuf| GrfError::NonUtf8Path {
        path: pathbuf.to_string_lossy().to_string(),
    })
}
