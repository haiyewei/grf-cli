use std::io;

use miette::Diagnostic;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GrfError>;

#[derive(Debug, Error, Diagnostic)]
pub enum GrfError {
    #[error("未找到 Git 可执行文件")]
    #[diagnostic(
        code(grf::git::not_found),
        help("请先安装 Git，并确保 `git` 在 PATH 中可执行。")
    )]
    GitNotFound,

    #[error("Git 命令执行失败: {command}")]
    #[diagnostic(code(grf::git::command_failed))]
    GitCommandFailed { command: String, stderr: String },

    #[error("仓库不存在: {name}")]
    #[diagnostic(code(grf::repo::not_found))]
    RepoNotFound { name: String },

    #[error("仓库已存在: {name}")]
    #[diagnostic(code(grf::repo::already_exists))]
    RepoAlreadyExists { name: String },

    #[error("仓库引用不唯一: {name}")]
    #[diagnostic(code(grf::repo::ambiguous))]
    AmbiguousRepoName { name: String, matches: Vec<String> },

    #[error("无效的 Git URL: {url}")]
    #[diagnostic(code(grf::repo::invalid_git_url))]
    InvalidGitUrl { url: String },

    #[error("路径不存在: {path}")]
    #[diagnostic(code(grf::fs::path_not_found))]
    PathNotFound { path: String },

    #[error("仓库 `{repo}` 中不存在子路径 `{subdir}`")]
    #[diagnostic(code(grf::repo::missing_subdir))]
    MissingSubdir { repo: String, subdir: String },

    #[error("当前没有缓存仓库")]
    #[diagnostic(
        code(grf::repo::empty),
        help("先运行 `grf add <url>` 添加一个参考仓库。")
    )]
    NoRepositories,

    #[error("当前工作区没有已加载的参考代码")]
    #[diagnostic(
        code(grf::load::empty),
        help("先运行 `grf load <name>` 把缓存仓库加载到当前项目。")
    )]
    NoLoadedReferences,

    #[error("读取 JSON 失败: {path}")]
    #[diagnostic(code(grf::state::json_parse))]
    JsonParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("读取文件失败: {path}")]
    #[diagnostic(code(grf::fs::read))]
    Io {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("写入文件失败: {path}")]
    #[diagnostic(code(grf::fs::write))]
    WriteIo {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("无法定位用户主目录")]
    #[diagnostic(code(grf::paths::home_unavailable))]
    HomeDirUnavailable,

    #[error("路径不是有效的 UTF-8: {path}")]
    #[diagnostic(code(grf::paths::non_utf8))]
    NonUtf8Path { path: String },

    #[error("参数无效: {message}")]
    #[diagnostic(code(grf::cli::invalid_argument))]
    InvalidArgument { message: String },

    #[error("操作已取消")]
    #[diagnostic(code(grf::cli::cancelled))]
    Cancelled,
}

impl GrfError {
    pub fn io_read(path: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn io_write(path: impl Into<String>, source: io::Error) -> Self {
        Self::WriteIo {
            path: path.into(),
            source,
        }
    }
}
