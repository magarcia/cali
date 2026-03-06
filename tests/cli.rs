use assert_cmd::Command;
use tempfile::TempDir;

#[allow(deprecated)]
fn cali_command() -> Command {
    Command::cargo_bin("cali").unwrap()
}

#[test]
fn test_shows_version() {
    cali_command().arg("--version").assert().success();
}

#[test]
fn test_shows_help() {
    cali_command().arg("--help").assert().success();
}

#[test]
fn test_config_list_no_config() {
    // Use a temp directory to ensure no config exists
    let temp_dir = TempDir::new().unwrap();
    cali_command()
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .args(["source", "list"])
        .assert()
        .failure();
}

#[test]
fn test_shows_usage_for_invalid_date() {
    // Invalid date format should fail
    cali_command()
        .args(["--date", "not-a-real-date-format-xyz"])
        .assert()
        .failure();
}
