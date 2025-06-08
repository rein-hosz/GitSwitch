use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;
use std::fs;
use std::path::Path;

fn setup_git_repo(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("git")
        .args(&["init"])
        .current_dir(repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(&["remote", "add", "origin", "https://github.com/user/repo.git"])
        .current_dir(repo_path)
        .assert()
        .success();
    Ok(())
}

#[test]
fn test_add_account_and_list() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let ssh_key_path = temp_dir.path().join(".ssh/id_rsa_testacc");
    let ssh_key_path_str = ssh_key_path.to_str().unwrap();

    // Create dummy SSH key files for the test FIRST
    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    fs::File::create(&ssh_key_path)?;
    fs::File::create(ssh_key_path.with_extension("pub"))?; // Also create .pub for completeness

    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_dir.path()); // Ensure config is in temp_dir

    // Add account
    cmd.args(&[
        "add",
        "testacc",
        "testuser",
        "testuser@example.com",
        "--ssh-key-path",
        ssh_key_path_str,
    ]);
    cmd.assert().success().stdout(predicate::str::contains("Account 'testacc' added successfully!"));

    // List accounts
    let mut cmd_list = Command::cargo_bin("git-switch")?;
    cmd_list.env("HOME", temp_dir.path());
    cmd_list.arg("list");
    cmd_list
        .assert()
        .success()
        .stdout(predicate::str::contains("testacc"))
        .stdout(predicate::str::contains("testuser"))
        .stdout(predicate::str::contains("testuser@example.com"))
        .stdout(predicate::str::contains("id_rsa_testacc"));

    Ok(())
}

#[test]
fn test_use_account_globally() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    
    let ssh_key_path_globalacc = temp_dir.path().join(".ssh/id_rsa_globalacc");
    let ssh_key_path_globalacc_str = ssh_key_path_globalacc.to_str().unwrap();

    // Create dummy SSH key files FIRST
    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    fs::File::create(&ssh_key_path_globalacc)?;
    // fs::File::create(ssh_key_path_globalacc.with_extension("pub"))?; // Optional: create .pub if needed by add logic

    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_dir.path());

    // Add account first
    cmd.args(&[
        "add",
        "globalacc",
        "globaluser",
        "global@example.com",
        "--ssh-key-path",
        ssh_key_path_globalacc_str,
    ]).assert().success();
    
    // Use account globally
    let mut cmd_use = Command::cargo_bin("git-switch")?;
    cmd_use.env("HOME", temp_dir.path());
    cmd_use.args(&["use", "globalacc"]);
    cmd_use.assert().success().stdout(predicate::str::contains("Global Git user.name and user.email set."));

    // Verify global git config (requires git command)
    let mut git_cmd = Command::new("git");
    git_cmd.env("HOME", temp_dir.path()); // Important for git to read the correct global .gitconfig
    git_cmd.args(&["config", "--global", "user.name"]);
    git_cmd.assert().success().stdout(predicate::str::contains("globaluser"));

    let mut git_cmd_email = Command::new("git");
    git_cmd_email.env("HOME", temp_dir.path());
    git_cmd_email.args(&["config", "--global", "user.email"]);
    git_cmd_email.assert().success().stdout(predicate::str::contains("global@example.com"));

    Ok(())
}

#[test]
fn test_account_subcommand_local_repo() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?; // Overall temp dir for config
    let repo_dir = tempdir()?; // Separate temp dir for git repo

    setup_git_repo(repo_dir.path())?;

    let ssh_key_path_localacc = temp_dir.path().join(".ssh/id_rsa_localacc");
    let ssh_key_path_localacc_str = ssh_key_path_localacc.to_str().unwrap();

    // Create dummy SSH key files FIRST
    let ssh_dir_config = temp_dir.path().join(".ssh"); // SSH keys related to git-switch config
    fs::create_dir_all(&ssh_dir_config)?;
    fs::File::create(&ssh_key_path_localacc)?;

    let mut cmd_add = Command::cargo_bin("git-switch")?;
    cmd_add.env("HOME", temp_dir.path());
    cmd_add.args(&[
        "add",
        "localacc",
        "localuser",
        "local@example.com",
        "--ssh-key-path",
        ssh_key_path_localacc_str,
    ]).assert().success();

    // Use account for the local repository
    let mut cmd_account = Command::cargo_bin("git-switch")?;
    cmd_account.env("HOME", temp_dir.path()); // git-switch reads its config from here
    cmd_account.current_dir(repo_dir.path()); // Run in the context of the repo
    cmd_account.args(&["account", "localacc"]);
    cmd_account.assert().success().stdout(predicate::str::contains("Git user.name, user.email, and core.sshCommand set locally for this repository."));

    // Verify local git config
    let mut git_cmd_name = Command::new("git");
    git_cmd_name.current_dir(repo_dir.path());
    git_cmd_name.args(&["config", "user.name"]);
    git_cmd_name.assert().success().stdout(predicate::str::contains("localuser"));

    let mut git_cmd_email = Command::new("git");
    git_cmd_email.current_dir(repo_dir.path());
    git_cmd_email.args(&["config", "user.email"]);
    git_cmd_email.assert().success().stdout(predicate::str::contains("local@example.com"));
    
    let mut git_cmd_ssh = Command::new("git");
    git_cmd_ssh.current_dir(repo_dir.path());
    git_cmd_ssh.args(&["config", "core.sshCommand"]);
    git_cmd_ssh.assert().success().stdout(predicate::str::contains("id_rsa_localacc"));


    Ok(())
}

#[test]
fn test_remote_https_to_ssh() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?; // For git-switch config if needed, though not directly for this
    let repo_dir = tempdir()?;
    setup_git_repo(repo_dir.path())?; // Sets up with https://github.com/user/repo.git

    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_dir.path());
    cmd.current_dir(repo_dir.path());
    cmd.args(&["remote", "--ssh"]);
    cmd.assert().success().stdout(predicate::str::contains("Remote 'origin' URL updated to: git@github.com:user/repo.git"));

    // Verify remote URL
    let mut git_cmd = Command::new("git");
    git_cmd.current_dir(repo_dir.path());
    git_cmd.args(&["remote", "get-url", "origin"]);
    git_cmd.assert().success().stdout(predicate::str::contains("git@github.com:user/repo.git"));
    
    Ok(())
}

#[test]
fn test_remote_ssh_to_https() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let repo_dir = tempdir()?;
    setup_git_repo(repo_dir.path())?;
    
    // Change remote to SSH first
    Command::new("git")
        .args(&["remote", "set-url", "origin", "git@github.com:user/another.git"])
        .current_dir(repo_dir.path())
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_dir.path());
    cmd.current_dir(repo_dir.path());
    cmd.args(&["remote", "--https"]);
    cmd.assert().success().stdout(predicate::str::contains("Remote 'origin' URL updated to: https://github.com/user/another.git"));

    // Verify remote URL
    let mut git_cmd = Command::new("git");
    git_cmd.current_dir(repo_dir.path());
    git_cmd.args(&["remote", "get-url", "origin"]);
    git_cmd.assert().success().stdout(predicate::str::contains("https://github.com/user/another.git"));

    Ok(())
}


#[test]
fn test_whoami_no_repo_global_set() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;

    let ssh_key_path_whoami_global = temp_dir.path().join(".ssh/id_rsa_whoami_global");
    let ssh_key_path_whoami_global_str = ssh_key_path_whoami_global.to_str().unwrap();

    // Create dummy SSH key files FIRST
    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    fs::File::create(&ssh_key_path_whoami_global)?;

    let mut cmd_add = Command::cargo_bin("git-switch")?;
    cmd_add.env("HOME", temp_dir.path());
    cmd_add.args(&[
        "add",
        "globalacc", // Account name used for 'use' later
        "globaluser",
        "global@example.com",
        "--ssh-key-path",
        ssh_key_path_whoami_global_str, // Path for the key of 'globalacc'
    ]).assert().success();
    
    let mut cmd_use = Command::cargo_bin("git-switch")?;
    cmd_use.env("HOME", temp_dir.path());
    cmd_use.args(&["use", "globalacc"]).assert().success();

    let mut cmd_whoami = Command::cargo_bin("git-switch")?;
    cmd_whoami.env("HOME", temp_dir.path());
    cmd_whoami.current_dir(temp_dir.path()); // Ensure command runs in a non-git directory
    cmd_whoami.arg("whoami");
    cmd_whoami.assert()
        .success()
        .stdout(predicate::str::contains("Not inside a Git repository."))
        .stdout(predicate::str::contains("Global Git Identity: globaluser <global@example.com>"))
        .stdout(predicate::str::contains("Effective git-switch Account: 'globalacc'"));
    Ok(())
}

#[test]
fn test_whoami_in_repo_local_set() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?; // For git-switch config
    let repo_dir = tempdir()?; // For git repo
    setup_git_repo(repo_dir.path())?;

    let ssh_key_path_localwhoami = temp_dir.path().join(".ssh/id_rsa_localwhoami");
    let ssh_key_path_localwhoami_str = ssh_key_path_localwhoami.to_str().unwrap();

    // Create dummy SSH key files FIRST
    let ssh_dir_config = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir_config)?;
    fs::File::create(&ssh_key_path_localwhoami)?;

    // Add account to git-switch
    let mut cmd_add = Command::cargo_bin("git-switch")?;
    cmd_add.env("HOME", temp_dir.path());
    cmd_add.args(&[
        "add",
        "localwhoami",
        "localiam",
        "locali@example.com",
        "--ssh-key-path",
        ssh_key_path_localwhoami_str,
    ]).assert().success();

    // Set local git config using git-switch account command
    let mut cmd_account = Command::cargo_bin("git-switch")?;
    cmd_account.env("HOME", temp_dir.path());
    cmd_account.current_dir(repo_dir.path());
    cmd_account.args(&["account", "localwhoami"]).assert().success();
    
    // Run whoami
    let mut cmd_whoami = Command::cargo_bin("git-switch")?;
    cmd_whoami.env("HOME", temp_dir.path());
    cmd_whoami.current_dir(repo_dir.path());
    cmd_whoami.arg("whoami");
    cmd_whoami.assert()
        .success()
        .stdout(predicate::str::contains("Local Repository Identity: localiam <locali@example.com>"))
        .stdout(predicate::str::contains("Linked to git-switch account: 'localwhoami'"))
        .stdout(predicate::str::contains("Remote 'origin' URL: https://github.com/user/repo.git"))
        .stdout(predicate::str::contains("Effective git-switch Account: 'localwhoami'"));
    Ok(())
}


#[test]
fn test_auth_test_with_specific_account() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let mut cmd_add = Command::cargo_bin("git-switch")?;
    cmd_add.env("HOME", temp_dir.path());

    // For this test to actually pass against github/gitlab, a real, configured key would be needed.
    // We are primarily testing that the command attempts to use the specified key.
    // We'll mock the key file.
    let mock_key_name = "test_auth_key";
    let mock_key_path = temp_dir.path().join(".ssh").join(mock_key_name);
    fs::create_dir_all(mock_key_path.parent().unwrap())?;
    fs::write(&mock_key_path, "mock ssh key content")?;
    
    cmd_add.args(&[
        "add",
        "authacc",
        "authuser",
        "auth@example.com",
        "--ssh-key-path",
        mock_key_path.to_str().unwrap(),
    ]).assert().success();

    let mut cmd_auth = assert_cmd::Command::cargo_bin("git-switch")?; // Use assert_cmd::Command for write_stdin
    cmd_auth.env("HOME", temp_dir.path());
    // Simulate selecting the account by number (assuming it's the first and only one)
    cmd_auth.args(&["auth", "test"]); 
    // Provide input "1" to select the first account
    cmd_auth.write_stdin("1\n");


    // We expect it to try connecting, which will likely fail for a mock key,
    // but the command should report its attempt.
    // The key part is that it mentions the key it's trying to use.
    cmd_auth.assert()
        .success() // The command itself should succeed even if auth fails
        .stdout(predicate::str::contains(format!("testing with key: {}", mock_key_path.display()).as_str()))
        .stdout(predicate::str::contains("Attempting SSH authentication test..."))
        .stdout(predicate::str::contains("git@github.com"))
        .stdout(predicate::str::contains("git@gitlab.com"));
        // Depending on actual ssh client output, "Permission denied" or similar might appear in stderr
        // For a unit/integration test not hitting network, this is as far as we can reliably go.
    Ok(())
}

#[test]
fn test_remove_account() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    
    let key_path_str = temp_dir.path().join(".ssh/id_rsa_removeacc").to_str().unwrap().to_string();
    let pub_key_path_str = format!("{}.pub", key_path_str);

    // Create dummy SSH key files FIRST
    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    fs::File::create(&key_path_str)?;
    fs::File::create(&pub_key_path_str)?;

    // Add account
    let mut cmd_add = Command::cargo_bin("git-switch")?;
    cmd_add.env("HOME", temp_dir.path());
    cmd_add.args(&[
        "add",
        "removeacc",
        "removeuser",
        "remove@example.com",
        "--ssh-key-path",
        &key_path_str,
    ]);
    cmd_add.assert().success();

    // Remove account with prompt for key deletion (yes to both)
    let mut cmd_remove = assert_cmd::Command::cargo_bin("git-switch")?; // Correctly use assert_cmd::Command
    cmd_remove.env("HOME", temp_dir.path());
    cmd_remove.args(&["remove", "removeacc"]);
    cmd_remove.write_stdin("y\ny\n"); // Yes to remove account, Yes to delete key

    cmd_remove.assert()
        .success()
        .stdout(predicate::str::contains("Account 'removeacc' removed from configuration."))
        .stdout(predicate::str::contains("SSH private key file"))
        .stdout(predicate::str::contains("deleted"))
        .stdout(predicate::str::contains("SSH public key file"));


    // Verify key files are deleted
    assert!(!Path::new(&key_path_str).exists());
    assert!(!Path::new(&pub_key_path_str).exists());

    // List accounts to confirm removal
    let mut cmd_list = Command::cargo_bin("git-switch")?;
    cmd_list.env("HOME", temp_dir.path());
    cmd_list.arg("list");
    cmd_list.assert().success().stdout(predicate::str::contains("No saved accounts.").or(predicate::str::contains("Saved Git Accounts:").not()));


    Ok(())
}

#[test]
fn test_add_account_default_ssh_key_generation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let mut cmd = Command::cargo_bin("git-switch")?;
    cmd.env("HOME", temp_dir.path());

    // Add account without specifying SSH key path
    cmd.args(&["add", "defaultkeyacc", "defaultkeyuser", "defaultkey@example.com"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account 'defaultkeyacc' added successfully!"))
        .stdout(predicate::str::contains("Generating SSH key at"))
        .stdout(predicate::str::contains("id_rsa_defaultkeyacc")); // Check for default key name

    // Verify that SSH key files were created by the 'add' command
    let expected_key_path = temp_dir.path().join(".ssh/id_rsa_defaultkeyacc");
    let expected_pub_key_path = temp_dir.path().join(".ssh/id_rsa_defaultkeyacc.pub");
    
    // These assertions now correctly test if the application created the files.
    // The manual fs::create_dir_all and fs::File::create calls that were here previously
    // have been removed as they would mask whether ssh::generate_ssh_key worked.
    assert!(expected_key_path.exists(), "Private key file should be generated by the application");
    assert!(expected_pub_key_path.exists(), "Public key file should be generated by the application");

    // Check SSH config was updated
    let ssh_config_path = temp_dir.path().join(".ssh/config");
    assert!(ssh_config_path.exists());
    let ssh_config_content = fs::read_to_string(ssh_config_path)?;
    assert!(ssh_config_content.contains("Host github.com-defaultkeyacc"));
    assert!(ssh_config_content.contains("IdentityFile"));
    assert!(ssh_config_content.contains("id_rsa_defaultkeyacc"));

    Ok(())
}