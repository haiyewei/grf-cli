use std::fs::File;
use std::io::{BufReader, Write};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Serialize, de::DeserializeOwned};
use tempfile::NamedTempFile;
use walkdir::WalkDir;

use crate::domain::error::{GrfError, Result};

pub fn path_exists(path: &Utf8Path) -> bool {
    path.exists()
}

pub fn ensure_dir(path: &Utf8Path) -> Result<()> {
    fs_err::create_dir_all(path).map_err(|source| GrfError::io_write(path.to_string(), source))
}

pub fn remove_path(path: &Utf8Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs_err::remove_dir_all(path).map_err(|source| GrfError::io_write(path.to_string(), source))
    } else {
        fs_err::remove_file(path).map_err(|source| GrfError::io_write(path.to_string(), source))
    }
}

pub fn read_json_or_default<T>(path: &Utf8Path) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }

    let file = File::open(path).map_err(|source| GrfError::io_read(path.to_string(), source))?;
    serde_json::from_reader(BufReader::new(file)).map_err(|source| GrfError::JsonParse {
        path: path.to_string(),
        source,
    })
}

pub fn write_json_atomic<T>(path: &Utf8Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let parent = path.parent().ok_or_else(|| GrfError::InvalidArgument {
        message: format!("路径没有父目录: {path}"),
    })?;
    let mut temp = NamedTempFile::new_in(parent)
        .map_err(|source| GrfError::io_write(parent.to_string(), source))?;
    serde_json::to_writer_pretty(&mut temp, value).map_err(|source| GrfError::JsonParse {
        path: path.to_string(),
        source,
    })?;
    temp.write_all(b"\n")
        .map_err(|source| GrfError::io_write(path.to_string(), source))?;
    temp.persist(path)
        .map_err(|error| GrfError::io_write(path.to_string(), error.error))?;
    Ok(())
}

pub fn copy_path(src: &Utf8Path, dest: &Utf8Path, exclude_names: &[&str]) -> Result<()> {
    if !src.exists() {
        return Err(GrfError::PathNotFound {
            path: src.to_string(),
        });
    }

    if src.is_file() {
        if let Some(parent) = dest.parent() {
            ensure_dir(parent)?;
        }
        fs_err::copy(src, dest).map_err(|source| GrfError::io_write(dest.to_string(), source))?;
        return Ok(());
    }

    ensure_dir(dest)?;

    let walker = WalkDir::new(src).into_iter().filter_entry(|entry| {
        !exclude_names.contains(&entry.file_name().to_string_lossy().as_ref())
    });

    for entry in walker {
        let entry = entry.map_err(|source| GrfError::io_read(src.to_string(), source.into()))?;
        let entry_path =
            Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).map_err(|pathbuf| {
                GrfError::NonUtf8Path {
                    path: pathbuf.to_string_lossy().to_string(),
                }
            })?;

        if entry_path == src {
            continue;
        }

        let relative = entry_path
            .strip_prefix(src)
            .map_err(|_| GrfError::InvalidArgument {
                message: format!("无法计算相对路径: {entry_path}"),
            })?;
        let target = dest.join(relative);

        if entry.file_type().is_dir() {
            ensure_dir(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                ensure_dir(parent)?;
            }
            fs_err::copy(&entry_path, &target)
                .map_err(|source| GrfError::io_write(target.to_string(), source))?;
        }
    }

    Ok(())
}

pub fn read_dir(path: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
    let entries =
        fs_err::read_dir(path).map_err(|source| GrfError::io_read(path.to_string(), source))?;
    let mut paths = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|source| GrfError::io_read(path.to_string(), source))?;
        paths.push(Utf8PathBuf::from_path_buf(entry.path()).map_err(|pathbuf| {
            GrfError::NonUtf8Path {
                path: pathbuf.to_string_lossy().to_string(),
            }
        })?);
    }

    Ok(paths)
}
