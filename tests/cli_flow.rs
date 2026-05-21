use std::path::Path;
use std::process::Command as ProcessCommand;

use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;
use serde_json::Value;

fn grf_cmd(state_root: &Path, cwd: &Path) -> Command {
    let mut cmd = Command::cargo_bin("grf").unwrap();
    cmd.current_dir(cwd);
    cmd.env("GRF_ROOT_DIR", state_root);
    cmd.env("NO_COLOR", "1");
    cmd
}

#[test]
fn add_list_load_unload_clean_flow_with_local_repo() {
    let temp = TempDir::new().unwrap();
    let state_root = temp.path().join("state");
    let workspace = temp.path().join("workspace");
    let source_repo = temp.path().join("source-repo");
    std::fs::create_dir_all(&workspace).unwrap();

    init_git_repo(&source_repo);
    write_file(&source_repo.join("README.md"), "hello from source\n");
    git(&["add", "."], &source_repo);
    git(&["commit", "-m", "initial"], &source_repo);

    grf_cmd(&state_root, &workspace)
        .args(["add", source_repo.to_str().unwrap(), "--name", "sample"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added sample"));

    let list_output = grf_cmd(&state_root, &workspace)
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let repos: Value = serde_json::from_slice(&list_output).unwrap();
    assert_eq!(repos.as_array().unwrap().len(), 1);
    assert_eq!(repos[0]["name"], "sample");

    grf_cmd(&state_root, &workspace)
        .args(["load", "sample", "vendor/reference"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded sample"));

    let loaded_file = workspace.join("vendor").join("reference").join("README.md");
    assert!(loaded_file.exists());
    assert_eq!(
        normalize_newlines(&std::fs::read_to_string(&loaded_file).unwrap()),
        "hello from source\n"
    );

    let load_list_output = grf_cmd(&state_root, &workspace)
        .args(["list", "--load", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let entries: Value = serde_json::from_slice(&load_list_output).unwrap();
    assert_eq!(entries.as_array().unwrap().len(), 1);
    assert_eq!(entries[0]["repo_name"], "sample");
    assert_eq!(entries[0]["target_path"], "vendor/reference");

    let gitignore = workspace.join(".gitignore");
    let gitignore_content = std::fs::read_to_string(&gitignore).unwrap();
    assert!(gitignore_content.contains(".gitreference/"));
    assert!(gitignore_content.contains("vendor/reference/"));

    grf_cmd(&state_root, &workspace)
        .args(["unload", "sample", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));

    assert!(!loaded_file.exists());
    let gitignore_after_unload = std::fs::read_to_string(&gitignore).unwrap_or_default();
    assert!(!gitignore_after_unload.contains("vendor/reference/"));
    assert!(!gitignore_after_unload.contains(".gitreference/"));

    grf_cmd(&state_root, &workspace)
        .args(["clean", "sample", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed sample"));

    let final_list_output = grf_cmd(&state_root, &workspace)
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let final_repos: Value = serde_json::from_slice(&final_list_output).unwrap();
    assert_eq!(final_repos.as_array().unwrap().len(), 0);
}

#[test]
fn update_sync_refreshes_workspace_from_local_remote() {
    let temp = TempDir::new().unwrap();
    let state_root = temp.path().join("state");
    let workspace = temp.path().join("workspace");
    let source_repo = temp.path().join("source-repo");
    std::fs::create_dir_all(&workspace).unwrap();

    init_git_repo(&source_repo);
    write_file(&source_repo.join("notes.txt"), "version-one\n");
    git(&["add", "."], &source_repo);
    git(&["commit", "-m", "initial"], &source_repo);

    grf_cmd(&state_root, &workspace)
        .args(["add", source_repo.to_str().unwrap(), "--name", "sample"])
        .assert()
        .success();

    grf_cmd(&state_root, &workspace)
        .args(["load", "sample", "vendor/reference"])
        .assert()
        .success();

    write_file(&source_repo.join("notes.txt"), "version-two\n");
    git(&["add", "."], &source_repo);
    git(&["commit", "-m", "update"], &source_repo);

    grf_cmd(&state_root, &workspace)
        .args(["update", "sample", "--sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated").and(predicate::str::contains("synced")));

    let synced_file = workspace.join("vendor").join("reference").join("notes.txt");
    assert_eq!(
        normalize_newlines(&std::fs::read_to_string(&synced_file).unwrap()),
        "version-two\n"
    );

    grf_cmd(&state_root, &workspace)
        .args(["update", "--status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("in sync"));
}

#[test]
fn add_recovers_orphaned_cached_repo_and_allows_load() {
    let temp = TempDir::new().unwrap();
    let state_root = temp.path().join("state");
    let workspace = temp.path().join("workspace");
    let source_repo = temp.path().join("source-repo");
    std::fs::create_dir_all(&workspace).unwrap();

    init_git_repo(&source_repo);
    write_file(&source_repo.join("README.md"), "hello from source\n");
    git(&["add", "."], &source_repo);
    git(&["commit", "-m", "initial"], &source_repo);

    let repo_name = "github.com/openai/codex";

    grf_cmd(&state_root, &workspace)
        .args(["add", source_repo.to_str().unwrap(), "--name", repo_name])
        .assert()
        .success();

    let config_path = state_root.join("config.v2.json");
    let mut config: Value = serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
    config["repos"] = serde_json::json!({});
    std::fs::write(
        &config_path,
        format!("{}\n", serde_json::to_string_pretty(&config).unwrap()),
    )
    .unwrap();

    grf_cmd(&state_root, &workspace)
        .args(["add", source_repo.to_str().unwrap(), "--name", repo_name])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added github.com/openai/codex"));

    grf_cmd(&state_root, &workspace)
        .args(["load", repo_name, "vendor/reference"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded github.com/openai/codex"));
}

#[test]
fn new_state_files_ignore_legacy_json_names() {
    let temp = TempDir::new().unwrap();
    let state_root = temp.path().join("state");
    let workspace = temp.path().join("workspace");
    let source_repo = temp.path().join("source-repo");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&state_root).unwrap();

    write_file(
        &state_root.join("config.json"),
        "{\n  \"repos\": {\n    \"legacy\": {\"url\": \"x\"}\n  }\n}\n",
    );
    write_file(
        &state_root.join("loading.json"),
        "{\n  \"loadedRepos\": {}\n}\n",
    );

    init_git_repo(&source_repo);
    write_file(&source_repo.join("README.md"), "hello from source\n");
    git(&["add", "."], &source_repo);
    git(&["commit", "-m", "initial"], &source_repo);

    grf_cmd(&state_root, &workspace)
        .args(["add", source_repo.to_str().unwrap(), "--name", "sample"])
        .assert()
        .success();

    grf_cmd(&state_root, &workspace)
        .args(["load", "sample", "vendor/reference"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded sample"));

    let config_v2 = state_root.join("config.v2.json");
    let loading_v2 = state_root.join("loading.v2.json");
    assert!(config_v2.exists());
    assert!(loading_v2.exists());

    let loading_json: Value = serde_json::from_slice(&std::fs::read(&loading_v2).unwrap()).unwrap();
    assert_eq!(loading_json["version"], "2.0.0");
    assert_eq!(loading_json["entries"].as_array().unwrap().len(), 1);

    let legacy_loading = std::fs::read_to_string(state_root.join("loading.json")).unwrap();
    assert!(legacy_loading.contains("\"loadedRepos\""));
}

#[test]
fn load_uses_existing_cached_repo_when_given_local_path_again() {
    let temp = TempDir::new().unwrap();
    let state_root = temp.path().join("state");
    let workspace = temp.path().join("workspace");
    let source_repo = temp.path().join("source-repo");
    std::fs::create_dir_all(&workspace).unwrap();

    init_git_repo(&source_repo);
    write_file(&source_repo.join("README.md"), "hello from source\n");
    git(&["add", "."], &source_repo);
    git(&["commit", "-m", "initial"], &source_repo);

    grf_cmd(&state_root, &workspace)
        .args(["add", source_repo.to_str().unwrap()])
        .assert()
        .success();

    grf_cmd(&state_root, &workspace)
        .args(["load", source_repo.to_str().unwrap(), "vendor/reference"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded"));
}

fn init_git_repo(path: &Path) {
    std::fs::create_dir_all(path).unwrap();
    git(&["init"], path);
    git(&["config", "user.name", "grf-tests"], path);
    git(&["config", "user.email", "grf-tests@example.com"], path);
}

fn git(args: &[&str], cwd: &Path) {
    let status = ProcessCommand::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "git command failed: git {}",
        args.join(" ")
    );
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n")
}
