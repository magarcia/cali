use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[allow(deprecated)]
fn cali_command() -> Command {
    Command::cargo_bin("cali").unwrap()
}

fn isolated_command() -> (Command, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = cali_command();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .env("XDG_CACHE_HOME", temp_dir.path());
    (cmd, temp_dir)
}

// --- Version & Help ---

#[test]
fn test_shows_version_with_cali_prefix() {
    cali_command()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("cali "));
}

#[test]
fn test_help_contains_subcommands() {
    cali_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("source"))
        .stdout(predicate::str::contains("sync"))
        .stdout(predicate::str::contains("config"));
}

#[test]
fn test_source_help() {
    cali_command()
        .args(["source", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("list"));
}

// --- Error handling & exit codes ---

#[test]
fn test_source_list_no_config_shows_error_message() {
    let (mut cmd, _dir) = isolated_command();
    cmd.args(["source", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Configuration not found"))
        .code(3);
}

#[test]
fn test_invalid_date_exits_with_code_2() {
    let (mut cmd, _dir) = isolated_command();
    cmd.arg("not-a-real-date-format-xyz")
        .assert()
        .failure()
        .code(predicate::ne(0));
}

#[test]
fn test_no_config_shows_onboarding_hint() {
    let (mut cmd, _dir) = isolated_command();
    cmd.args(["source", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cali"));
}

// --- Output format ---

#[test]
fn test_output_json_flag_accepted() {
    cali_command()
        .args(["--output", "json", "--help"])
        .assert()
        .success();
}

#[test]
fn test_output_llm_flag_accepted() {
    cali_command()
        .args(["--output", "llm", "--help"])
        .assert()
        .success();
}

#[test]
fn test_invalid_output_format_rejected() {
    cali_command()
        .args(["--output", "xml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// --- Global flags ---

#[test]
fn test_no_color_flag_accepted() {
    cali_command()
        .args(["--no-color", "--help"])
        .assert()
        .success();
}

#[test]
fn test_verbose_and_quiet_conflict() {
    cali_command()
        .args(["--verbose", "--quiet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

// --- Completions ---

#[test]
fn test_completions_zsh() {
    cali_command()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("compdef"));
}

#[test]
fn test_completions_bash() {
    cali_command()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

// --- Grep flag ---

#[test]
fn test_grep_flag_accepted() {
    cali_command()
        .args(["--grep", "standup", "--help"])
        .assert()
        .success();
}
