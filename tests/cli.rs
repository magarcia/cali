use assert_cmd::Command;

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
fn test_config_list_runs() {
    // Just check that the command runs (with or without config)
    cali_command().args(["config", "list"]).assert().success();
}

#[test]
fn test_shows_usage_for_invalid_date() {
    // Invalid date format should fail
    cali_command()
        .args(["--date", "not-a-real-date-format-xyz"])
        .assert()
        .failure();
}
