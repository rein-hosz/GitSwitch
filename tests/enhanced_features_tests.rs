use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_template_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_home);
    cmd.args(&["template", "list"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available account templates"))
        .stdout(predicate::str::contains("github"))
        .stdout(predicate::str::contains("gitlab"));

    Ok(())
}

#[test]
fn test_template_account_creation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_home);
    cmd.args(&["template", "use", "github", "test-account", "Test User", "test@example.com"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account 'test-account' created from github template"));

    // Verify the account was created with correct details
    let mut list_cmd = Command::cargo_bin("git-switch")?;
    list_cmd.env("HOME", temp_home);
    list_cmd.args(&["list", "--detailed"]);
    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-account"))
        .stdout(predicate::str::contains("Provider: github"))
        .stdout(predicate::str::contains("~/.ssh/id_rsa_github"));

    Ok(())
}

#[test]
fn test_backup_and_restore() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    // Create an account first
    let mut add_cmd = Command::cargo_bin("git-switch")?;
    add_cmd.env("HOME", temp_home);
    add_cmd.args(&["add", "backup-test", "Test User", "test@example.com"]);
    add_cmd.assert().success();

    // Create backup
    let mut backup_cmd = Command::cargo_bin("git-switch")?;
    backup_cmd.env("HOME", temp_home);
    backup_cmd.args(&["backup", "create"]);
    backup_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration backed up to"));

    // Verify backup file exists and is in TOML format
    let backup_path = temp_home.join("git-switch-backup.toml");
    assert!(backup_path.exists());
    
    let backup_content = fs::read_to_string(&backup_path)?;
    assert!(backup_content.contains("version = \"2.0\""));
    assert!(backup_content.contains("[accounts.backup-test]"));

    Ok(())
}

#[test]
fn test_analytics_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    // Test showing analytics when no data exists
    let mut analytics_cmd = Command::cargo_bin("git-switch")?;
    analytics_cmd.env("HOME", temp_home);
    analytics_cmd.args(&["analytics", "show"]);
    analytics_cmd.assert()
        .success()
        .stdout(predicate::str::contains("No usage data available yet"));

    Ok(())
}

#[test]
fn test_detection_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    // Test detection in non-git directory
    let mut detect_cmd = Command::cargo_bin("git-switch")?;
    detect_cmd.env("HOME", temp_home);
    detect_cmd.current_dir(temp_home);
    detect_cmd.args(&["detect"]);
    detect_cmd.assert()
        .success()
        .stdout(predicate::str::contains("No account detected"));

    Ok(())
}

#[test]
fn test_enhanced_list_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    // Test list when no accounts exist
    let mut list_cmd = Command::cargo_bin("git-switch")?;
    list_cmd.env("HOME", temp_home);
    list_cmd.args(&["list"]);
    list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("No accounts configured"));

    // Create an account and test detailed list
    let mut add_cmd = Command::cargo_bin("git-switch")?;
    add_cmd.env("HOME", temp_home);
    add_cmd.args(&["add", "list-test", "Test User", "test@example.com"]);
    add_cmd.assert().success();

    let mut detailed_list_cmd = Command::cargo_bin("git-switch")?;
    detailed_list_cmd.env("HOME", temp_home);
    detailed_list_cmd.args(&["list", "--detailed"]);
    detailed_list_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configured Accounts (Detailed)"))
        .stdout(predicate::str::contains("Username: Test User"))
        .stdout(predicate::str::contains("Email: test@example.com"));

    Ok(())
}

#[test]
fn test_colored_output_control() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    // Test with --no-color flag
    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_home);
    cmd.args(&["--no-color", "list"]);
    cmd.assert().success();

    // Test with colored output (default)
    let mut colored_cmd = Command::cargo_bin("git-switch")?;
    colored_cmd.env("HOME", temp_home);
    colored_cmd.args(&["list"]);
    colored_cmd.assert().success();

    Ok(())
}

#[test]
fn test_config_migration_to_toml() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    // Create a legacy JSON config file
    let json_config = r#"{
        "accounts": {
            "legacy-account": {
                "name": "legacy-account",
                "username": "Legacy User",
                "email": "legacy@example.com",
                "ssh_key_path": "~/.ssh/id_rsa_legacy"
            }
        }
    }"#;

    let legacy_config_path = temp_home.join(".git-switch-config.json");
    fs::write(&legacy_config_path, json_config)?;

    // Run any command that loads config (should trigger migration)
    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_home);
    cmd.args(&["list"]);
    cmd.assert().success();

    // Verify migration to TOML format
    let toml_config_path = temp_home.join(".git-switch-config.toml");
    assert!(toml_config_path.exists());

    let toml_content = fs::read_to_string(&toml_config_path)?;
    assert!(toml_content.contains("version = \"2.0\""));
    assert!(toml_content.contains("[accounts.legacy-account]"));

    // Verify backup of original JSON file
    let backup_path = temp_home.join(".git-switch-config.json.backup");
    assert!(backup_path.exists());

    Ok(())
}

#[test]
fn test_email_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home = temp_dir.path();

    // Test with invalid email
    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_home);
    cmd.args(&["add", "invalid-email", "Test User", "invalid-email"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid email format"));

    Ok(())
}
