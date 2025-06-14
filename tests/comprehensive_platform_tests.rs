use assert_cmd::Command as AssertCommand;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::tempdir;

// =============================================================================
// TEST OUTPUT HELPERS - Colorful UX and visibility
// =============================================================================

// ANSI Color codes for beautiful output
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

// Colors we're using
const CYAN: &str = "\x1b[36m";

// Bright colors we're using
const BRIGHT_BLACK: &str = "\x1b[90m";
const BRIGHT_RED: &str = "\x1b[91m";
const BRIGHT_GREEN: &str = "\x1b[92m";
const BRIGHT_YELLOW: &str = "\x1b[93m";
const BRIGHT_BLUE: &str = "\x1b[94m";
const BRIGHT_MAGENTA: &str = "\x1b[95m";
const BRIGHT_CYAN: &str = "\x1b[96m";
const BRIGHT_WHITE: &str = "\x1b[97m";

/// Print a prominent colorful test section header
fn print_test_header(test_name: &str, description: &str) {
    println!(
        "\n{}{}🧪 ═══════════════════════════════════════════════════════════{}",
        BOLD, BRIGHT_CYAN, RESET
    );
    println!(
        "{}{}   🔬 TEST: {}{}{}",
        BOLD,
        BRIGHT_WHITE,
        BRIGHT_YELLOW,
        test_name.to_uppercase(),
        RESET
    );
    println!(
        "{}{}   📝 DESC: {}{}{}",
        BOLD, BRIGHT_WHITE, BRIGHT_GREEN, description, RESET
    );
    println!(
        "{}{}   🖥️  PLATFORM: {}{}{}",
        BOLD,
        BRIGHT_WHITE,
        BRIGHT_MAGENTA,
        get_current_platform(),
        RESET
    );
    println!(
        "{}{}═══════════════════════════════════════════════════════════{}\n",
        BOLD, BRIGHT_CYAN, RESET
    );
}

/// Print colorful test step information
fn print_test_step(step: &str, details: &str) {
    println!(
        "{}{}📋 STEP {}: {}{}{}",
        BOLD, BRIGHT_BLUE, step, BRIGHT_WHITE, details, RESET
    );
}

/// Print colorful test success message
fn print_test_success(test_name: &str) {
    println!(
        "{}{}✅ SUCCESS: {}{}{} completed successfully{}",
        BOLD, BRIGHT_GREEN, BRIGHT_WHITE, test_name, BRIGHT_GREEN, RESET
    );
    println!(
        "{}{}─────────────────────────────────────────────────────────────{}\n",
        DIM, BRIGHT_BLACK, RESET
    );
}

/// Get current platform name for display with color
fn get_current_platform() -> &'static str {
    if cfg!(windows) {
        "🪟 Windows"
    } else if cfg!(target_os = "macos") {
        "🍎 macOS"
    } else if cfg!(target_os = "linux") {
        "🐧 Linux"
    } else {
        "🖥️ Unix"
    }
}

/// Print colorful command being executed
fn print_command_info(cmd_args: &[&str]) {
    println!(
        "{}{}🔧 COMMAND: {}git-switch {}{}{}",
        BOLD,
        BRIGHT_YELLOW,
        CYAN,
        cmd_args.join(" "),
        BRIGHT_YELLOW,
        RESET
    );
}

/// Print colorful test environment info
fn print_environment_info(temp_home: &Path) {
    println!(
        "{}{}🏠 TEST ENV: {}{}{}{}",
        BOLD,
        BRIGHT_MAGENTA,
        BRIGHT_WHITE,
        temp_home.display(),
        BRIGHT_MAGENTA,
        RESET
    );
}

/// Print colorful separator line
fn print_separator() {
    println!(
        "{}{}▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓{}",
        DIM, BRIGHT_BLACK, RESET
    );
}

/// Print colorful error message
fn print_error_expectation(message: &str) {
    println!(
        "{}{}⚠️  EXPECT ERROR: {}{}{}",
        BOLD, BRIGHT_RED, BRIGHT_WHITE, message, RESET
    );
}

// =============================================================================
// HELPER FUNCTIONS - Cross-platform support
// =============================================================================

/// Create git-switch command with cross-platform environment isolation
fn get_git_switch_command(
    temp_home_path: &Path,
) -> Result<AssertCommand, Box<dyn std::error::Error>> {
    let mut cmd = AssertCommand::cargo_bin("git-switch")?;

    // Set home directory based on platform
    if cfg!(windows) {
        cmd.env("USERPROFILE", temp_home_path);
    } else {
        cmd.env("HOME", temp_home_path);
    }

    // Remove git config interference
    cmd.env_remove("GIT_CONFIG_GLOBAL");
    cmd.env_remove("GIT_CONFIG_SYSTEM");
    cmd.env_remove("GIT_CONFIG_NOSYSTEM");

    Ok(cmd)
}

/// Create git command with cross-platform environment isolation
fn get_git_command(temp_home_path: &Path) -> StdCommand {
    let mut cmd = StdCommand::new("git");

    if cfg!(windows) {
        cmd.env("USERPROFILE", temp_home_path);
    } else {
        cmd.env("HOME", temp_home_path);
    }

    cmd.env_remove("GIT_CONFIG_GLOBAL");
    cmd.env_remove("GIT_CONFIG_SYSTEM");
    cmd.env_remove("GIT_CONFIG_NOSYSTEM");
    cmd
}

/// Setup a git repository for testing
fn setup_git_repo(
    repo_path: &Path,
    temp_home_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    get_git_command(temp_home_path)
        .args(&["init"])
        .current_dir(repo_path)
        .assert()
        .success();

    get_git_command(temp_home_path)
        .args(&["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .assert()
        .success();

    get_git_command(temp_home_path)
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .assert()
        .success();

    get_git_command(temp_home_path)
        .args(&[
            "remote",
            "add",
            "origin",
            "https://github.com/user/repo.git",
        ])
        .current_dir(repo_path)
        .assert()
        .success();

    Ok(())
}

/// Add test account for use in multiple tests
fn add_test_account(
    temp_home_path: &Path,
    name: &str,
    username: &str,
    email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["add", name, username, email]);
    cmd.assert().success();
    Ok(())
}

// =============================================================================
// CORE ACCOUNT MANAGEMENT TESTS
// =============================================================================

#[test]
fn test_add_account_basic() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header(
        "test_add_account_basic",
        "Testing basic account creation functionality",
    );

    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    print_environment_info(temp_home_path);
    print_test_step("1", "Creating new account with basic parameters");
    print_command_info(&["add", "test-basic", "basicuser", "basic@example.com"]);

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["add", "test-basic", "basicuser", "basic@example.com"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account Created Successfully"))
        .stdout(predicate::str::contains("test-basic"));

    print_test_success("test_add_account_basic");

    Ok(())
}

#[test]
fn test_add_account_with_spaces() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header(
        "test_add_account_with_spaces",
        "Testing account creation with spaces in account name",
    );

    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    print_environment_info(temp_home_path);
    print_test_step(
        "1",
        "Creating account with spaces in name (Test User Account)",
    );
    print_command_info(&["add", "Test User Account", "testuser", "test@example.com"]);

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["add", "Test User Account", "testuser", "test@example.com"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account Created Successfully"))
        .stdout(predicate::str::contains("Test User Account"));

    print_test_success("test_add_account_with_spaces");
    Ok(())
}

#[test]
fn test_add_account_with_provider() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Test each major provider
    let providers = vec![("github", "🐙"), ("gitlab", "🦊"), ("bitbucket", "🪣")];

    for (provider, emoji) in providers {
        let mut cmd = get_git_switch_command(temp_home_path)?;
        cmd.args(&[
            "add",
            &format!("{}-account", provider),
            &format!("{}user", provider),
            &format!("user@{}.com", provider),
            "--provider",
            provider,
        ]);

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Account Created Successfully"))
            .stdout(predicate::str::contains(emoji));
    }

    Ok(())
}

#[test]
fn test_list_accounts_empty() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["list"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No Git accounts configured"))
        .stdout(predicate::str::contains("git-switch add"));

    Ok(())
}

#[test]
fn test_list_accounts_simple() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Add some test accounts
    add_test_account(temp_home_path, "personal", "johndoe", "john@personal.com")?;
    add_test_account(temp_home_path, "work", "j.doe", "john.doe@work.com")?;

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["list"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("2 Accounts Configured"))
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("work"))
        .stdout(predicate::str::contains("✅")); // SSH key status

    Ok(())
}

#[test]
fn test_list_accounts_detailed() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Add test account with GitHub provider
    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&[
        "add",
        "github-test",
        "githubuser",
        "github@test.com",
        "--provider",
        "github",
    ]);
    cmd.assert().success();

    // Test detailed list
    let mut cmd_list = get_git_switch_command(temp_home_path)?;
    cmd_list.args(&["list", "--detailed"]);

    cmd_list
        .assert()
        .success()
        .stdout(predicate::str::contains("1 Account Configured"))
        .stdout(predicate::str::contains("github-test"))
        .stdout(predicate::str::contains("Username: githubuser"))
        .stdout(predicate::str::contains("Email: github@test.com"))
        .stdout(predicate::str::contains("Provider: GitHub"))
        .stdout(predicate::str::contains("SSH Key: Found"))
        .stdout(predicate::str::contains("git-switch use 'github-test'"));

    Ok(())
}

#[test]
fn test_use_account_globally() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Add account
    add_test_account(
        temp_home_path,
        "global-test",
        "globaluser",
        "global@test.com",
    )?;

    // Use account globally
    let mut cmd_use = get_git_switch_command(temp_home_path)?;
    cmd_use.args(&["use", "global-test"]);
    cmd_use
        .assert()
        .success()
        .stdout(predicate::str::contains("Global Git config updated"));

    // Verify global git config
    let mut git_cmd = get_git_command(temp_home_path);
    git_cmd.args(&["config", "--global", "user.name"]);
    git_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("globaluser"));

    let mut git_cmd_email = get_git_command(temp_home_path);
    git_cmd_email.args(&["config", "--global", "user.email"]);
    git_cmd_email
        .assert()
        .success()
        .stdout(predicate::str::contains("global@test.com"));

    Ok(())
}

#[test]
fn test_remove_account() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Add account
    add_test_account(
        temp_home_path,
        "remove-test",
        "removeuser",
        "remove@test.com",
    )?;

    // Remove account with prompts
    let mut cmd_remove = get_git_switch_command(temp_home_path)?;
    cmd_remove.args(&["remove", "remove-test", "--no-prompt"]);
    cmd_remove
        .assert()
        .success()
        .stdout(predicate::str::contains("Account 'remove-test' removed"));

    // Verify account is gone
    let mut cmd_list = get_git_switch_command(temp_home_path)?;
    cmd_list.args(&["list"]);
    cmd_list
        .assert()
        .success()
        .stdout(predicate::str::contains("No Git accounts configured"));

    Ok(())
}

// =============================================================================
// REPOSITORY-SPECIFIC TESTS
// =============================================================================

#[test]
fn test_account_subcommand_local_repo() -> Result<(), Box<dyn std::error::Error>> {
    let temp_config_dir = tempdir()?;
    let temp_home_path = temp_config_dir.path();
    let repo_dir = tempdir()?;

    setup_git_repo(repo_dir.path(), temp_home_path)?;
    add_test_account(
        temp_home_path,
        "local-account",
        "localuser",
        "local@test.com",
    )?;

    // Apply account to repository
    let mut cmd_account = get_git_switch_command(temp_home_path)?;
    cmd_account.current_dir(repo_dir.path());
    cmd_account.args(&["account", "local-account"]);
    cmd_account
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Repository configured for account 'local-account'",
        ));

    // Verify local git config
    let mut git_cmd = get_git_command(temp_home_path);
    git_cmd.current_dir(repo_dir.path());
    git_cmd.args(&["config", "user.name"]);
    git_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("localuser"));

    Ok(())
}

#[test]
fn test_remote_https_to_ssh() -> Result<(), Box<dyn std::error::Error>> {
    let temp_config_dir = tempdir()?;
    let temp_home_path = temp_config_dir.path();
    let repo_dir = tempdir()?;

    setup_git_repo(repo_dir.path(), temp_home_path)?;

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.current_dir(repo_dir.path());
    cmd.args(&["remote", "--ssh"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "Remote URL updated to: git@github.com:user/repo.git",
    ));

    Ok(())
}

#[test]
fn test_remote_ssh_to_https() -> Result<(), Box<dyn std::error::Error>> {
    let temp_config_dir = tempdir()?;
    let temp_home_path = temp_config_dir.path();
    let repo_dir = tempdir()?;

    setup_git_repo(repo_dir.path(), temp_home_path)?;

    // Set SSH URL first
    get_git_command(temp_home_path)
        .args(&[
            "remote",
            "set-url",
            "origin",
            "git@github.com:user/repo.git",
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success();

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.current_dir(repo_dir.path());
    cmd.args(&["remote", "--https"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "Remote URL updated to: https://github.com/user/repo.git",
    ));

    Ok(())
}

#[test]
fn test_whoami_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_config_dir = tempdir()?;
    let temp_home_path = temp_config_dir.path();
    let repo_dir = tempdir()?;

    setup_git_repo(repo_dir.path(), temp_home_path)?;
    add_test_account(
        temp_home_path,
        "whoami-test",
        "whoamiuser",
        "whoami@test.com",
    )?;

    // Set account for repository
    let mut cmd_account = get_git_switch_command(temp_home_path)?;
    cmd_account.current_dir(repo_dir.path());
    cmd_account.args(&["account", "whoami-test"]);
    cmd_account.assert().success();

    // Test whoami in repository
    let mut cmd_whoami = get_git_switch_command(temp_home_path)?;
    cmd_whoami.current_dir(repo_dir.path());
    cmd_whoami.args(&["whoami"]);
    cmd_whoami
        .assert()
        .success()
        .stdout(predicate::str::contains("whoamiuser"))
        .stdout(predicate::str::contains("whoami@test.com"));

    Ok(())
}

// =============================================================================
// TEMPLATE SYSTEM TESTS
// =============================================================================

#[test]
fn test_template_list() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    let mut cmd = get_git_switch_command(temp_home_path)?;
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
    let temp_home_path = temp_dir.path();

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&[
        "template",
        "use",
        "github",
        "template-test",
        "Template User",
        "template@test.com",
    ]);
    cmd.assert().success().stdout(predicate::str::contains(
        "Account 'template-test' created from github template",
    ));

    // Verify the account was created
    let mut list_cmd = get_git_switch_command(temp_home_path)?;
    list_cmd.args(&["list", "--detailed"]);
    list_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("template-test"))
        .stdout(predicate::str::contains("Provider: GitHub"));

    Ok(())
}

// =============================================================================
// AUTHENTICATION TESTS
// =============================================================================

#[test]
fn test_auth_test_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Add account with GitHub provider
    add_test_account(temp_home_path, "auth-test", "authuser", "auth@test.com")?;

    let mut cmd_auth = get_git_switch_command(temp_home_path)?;
    cmd_auth.args(&["auth", "test"]);
    cmd_auth
        .assert()
        .success()
        .stdout(predicate::str::contains("Testing SSH Authentication"));

    Ok(())
}

// =============================================================================
// BACKUP AND RESTORE TESTS
// =============================================================================

#[test]
fn test_backup_and_restore() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();
    let backup_file = temp_dir.path().join("backup.toml");

    // Add test accounts
    add_test_account(temp_home_path, "backup-test1", "user1", "user1@test.com")?;
    add_test_account(temp_home_path, "backup-test2", "user2", "user2@test.com")?;

    // Create backup
    let mut cmd_backup = get_git_switch_command(temp_home_path)?;
    cmd_backup.args(&[
        "backup",
        "create",
        "--output",
        backup_file.to_str().unwrap(),
    ]);
    cmd_backup
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration backed up"));

    // Verify backup file exists
    assert!(backup_file.exists());

    // Test restore
    let temp_restore_dir = tempdir()?;
    let temp_restore_home = temp_restore_dir.path();

    let mut cmd_restore = get_git_switch_command(temp_restore_home)?;
    cmd_restore.args(&["backup", "restore", backup_file.to_str().unwrap()]);
    cmd_restore
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration restored"));

    // Verify accounts were restored
    let mut cmd_list = get_git_switch_command(temp_restore_home)?;
    cmd_list.args(&["list"]);
    cmd_list
        .assert()
        .success()
        .stdout(predicate::str::contains("backup-test1"))
        .stdout(predicate::str::contains("backup-test2"));

    Ok(())
}

#[test]
fn test_export_import_accounts() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();
    let export_file = temp_dir.path().join("export.json");

    // Add test account
    add_test_account(
        temp_home_path,
        "export-test",
        "exportuser",
        "export@test.com",
    )?;

    // Export accounts
    let mut cmd_export = get_git_switch_command(temp_home_path)?;
    cmd_export.args(&[
        "backup",
        "export",
        export_file.to_str().unwrap(),
        "--format",
        "json",
    ]);
    cmd_export
        .assert()
        .success()
        .stdout(predicate::str::contains("Accounts exported"));

    assert!(export_file.exists());

    // Test import
    let temp_import_dir = tempdir()?;
    let temp_import_home = temp_import_dir.path();

    let mut cmd_import = get_git_switch_command(temp_import_home)?;
    cmd_import.args(&["backup", "import", export_file.to_str().unwrap()]);
    cmd_import
        .assert()
        .success()
        .stdout(predicate::str::contains("Accounts imported"));

    // Verify account was imported
    let mut cmd_list = get_git_switch_command(temp_import_home)?;
    cmd_list.args(&["list"]);
    cmd_list
        .assert()
        .success()
        .stdout(predicate::str::contains("export-test"));

    Ok(())
}

// =============================================================================
// REPOSITORY DISCOVERY TESTS
// =============================================================================

#[test]
fn test_repo_discover() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Create a test git repository
    let test_repo_dir = temp_dir.path().join("test-repo");
    fs::create_dir_all(&test_repo_dir)?;
    setup_git_repo(&test_repo_dir, temp_home_path)?;

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["repo", "discover", temp_dir.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Discovery Summary"));

    Ok(())
}

#[test]
fn test_repo_list() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["repo", "list"]);
    cmd.assert().success();

    Ok(())
}

// =============================================================================
// ANALYTICS TESTS
// =============================================================================

#[test]
fn test_analytics_commands() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Test analytics show
    let mut cmd_show = get_git_switch_command(temp_home_path)?;
    cmd_show.args(&["analytics", "show"]);
    cmd_show.assert().success();

    // Test analytics clear
    let mut cmd_clear = get_git_switch_command(temp_home_path)?;
    cmd_clear.args(&["analytics", "clear"]);
    cmd_clear.assert().success();

    Ok(())
}

// =============================================================================
// DETECTION AND SUGGESTIONS TESTS
// =============================================================================

#[test]
fn test_detect_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["detect"]);
    cmd.assert().success();

    Ok(())
}

// =============================================================================
// UTILITY COMMANDS TESTS
// =============================================================================

#[test]
fn test_completions_generation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Test different shells
    let shells = vec!["bash", "zsh", "fish", "powershell"];

    for shell in shells {
        let mut cmd = get_git_switch_command(temp_home_path)?;
        cmd.args(&["completions", shell]);
        cmd.assert().success();
    }

    Ok(())
}

#[test]
fn test_man_page_generation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();
    let man_dir = temp_dir.path().join("man");

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["man", "--output-dir", man_dir.to_str().unwrap()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated man page"));

    Ok(())
}

// =============================================================================
// PROFILE MANAGEMENT TESTS (Future functionality)
// =============================================================================

#[test]
fn test_profile_commands() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Test profile list (shows no profiles found message)
    let mut cmd_list = get_git_switch_command(temp_home_path)?;
    cmd_list.args(&["profile", "list"]);
    cmd_list
        .assert()
        .success()
        .stdout(predicate::str::contains("No profiles found"));

    Ok(())
}

// =============================================================================
// CROSS-PLATFORM VALIDATION TESTS
// =============================================================================

#[test]
fn test_platform_specific_home_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // This test ensures our cross-platform home directory handling works
    add_test_account(
        temp_home_path,
        "platform-test",
        "platformuser",
        "platform@test.com",
    )?;

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["list"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("platform-test"));

    Ok(())
}

#[test]
fn test_ssh_key_path_platform_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Add account and check SSH key was created with platform-appropriate path
    add_test_account(
        temp_home_path,
        "ssh-platform-test",
        "sshuser",
        "ssh@test.com",
    )?;

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["list", "--detailed"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("SSH Key: Found"))
        .stdout(predicate::str::contains("ssh-platform-test"));

    Ok(())
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

#[test]
fn test_error_duplicate_account() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header(
        "test_error_duplicate_account",
        "Testing error handling for duplicate account creation",
    );

    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    print_environment_info(temp_home_path);
    print_test_step("1", "Creating initial account");
    print_command_info(&["add", "duplicate-test", "user", "user@test.com"]);
    // Add account
    add_test_account(temp_home_path, "duplicate-test", "user", "user@test.com")?;

    print_test_step("2", "Attempting to create duplicate account (should fail)");
    print_error_expectation("This command should fail with 'already exists' error");
    print_command_info(&["add", "duplicate-test", "user2", "user2@test.com"]);
    // Try to add same account again
    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["add", "duplicate-test", "user2", "user2@test.com"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    print_test_success("test_error_duplicate_account");
    Ok(())
}

#[test]
fn test_error_account_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["use", "nonexistent-account"]);
    cmd.assert().failure().stderr(predicate::str::contains(
        "Account 'nonexistent-account' not found",
    ));

    Ok(())
}

#[test]
fn test_error_invalid_email() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["add", "invalid-email-test", "user", "invalid-email"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid email"));

    Ok(())
}

// =============================================================================
// INTEGRATION TESTS - Real workflow scenarios
// =============================================================================

#[test]
fn test_complete_workflow_personal_work() -> Result<(), Box<dyn std::error::Error>> {
    print_test_header(
        "test_complete_workflow_personal_work",
        "Complete workflow: managing separate work/personal Git identities",
    );

    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();
    let work_repo = temp_dir.path().join("work-project");
    let personal_repo = temp_dir.path().join("personal-project");

    print_environment_info(temp_home_path);
    print_separator();
    print_test_step("1", "Setting up work and personal repositories");

    // Setup repositories
    fs::create_dir_all(&work_repo)?;
    fs::create_dir_all(&personal_repo)?;
    setup_git_repo(&work_repo, temp_home_path)?;
    setup_git_repo(&personal_repo, temp_home_path)?;

    print_separator();
    print_test_step("2", "Creating work and personal accounts");
    print_command_info(&["add", "work", "John Doe", "john.doe@company.com"]);
    // Add work and personal accounts
    add_test_account(temp_home_path, "work", "John Doe", "john.doe@company.com")?;
    print_command_info(&["add", "personal", "John", "john@personal.com"]);
    add_test_account(temp_home_path, "personal", "John", "john@personal.com")?;

    print_separator();
    print_test_step("3", "Configuring work repository with work account");
    print_command_info(&["account", "work"]);
    // Configure work repository
    let mut cmd_work = get_git_switch_command(temp_home_path)?;
    cmd_work.current_dir(&work_repo);
    cmd_work.args(&["account", "work"]);
    cmd_work.assert().success();

    print_separator();
    print_test_step("4", "Configuring personal repository with personal account");
    print_command_info(&["account", "personal"]);
    // Configure personal repository
    let mut cmd_personal = get_git_switch_command(temp_home_path)?;
    cmd_personal.current_dir(&personal_repo);
    cmd_personal.args(&["account", "personal"]);
    cmd_personal.assert().success();

    print_separator();
    print_test_step("5", "Verifying repository configurations");
    // Verify configurations
    let mut git_work = get_git_command(temp_home_path);
    git_work.current_dir(&work_repo);
    git_work.args(&["config", "user.email"]);
    git_work
        .assert()
        .success()
        .stdout(predicate::str::contains("john.doe@company.com"));

    let mut git_personal = get_git_command(temp_home_path);
    git_personal.current_dir(&personal_repo);
    git_personal.args(&["config", "user.email"]);
    git_personal
        .assert()
        .success()
        .stdout(predicate::str::contains("john@personal.com"));

    print_test_success("test_complete_workflow_personal_work");
    Ok(())
}

#[test]
fn test_account_switching_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    // Add multiple accounts
    add_test_account(temp_home_path, "account1", "user1", "user1@test.com")?;
    add_test_account(temp_home_path, "account2", "user2", "user2@test.com")?;
    add_test_account(temp_home_path, "account3", "user3", "user3@test.com")?;

    // Use account1 globally
    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["use", "account1"]);
    cmd.assert().success();

    // Switch to account2
    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.args(&["use", "account2"]);
    cmd.assert().success();

    // Verify current global config
    let mut git_cmd = get_git_command(temp_home_path);
    git_cmd.args(&["config", "--global", "user.email"]);
    git_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("user2@test.com"));

    Ok(())
}
