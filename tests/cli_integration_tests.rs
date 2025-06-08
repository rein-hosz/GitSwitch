use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command as StdCommand; // Renamed to avoid conflict
use assert_cmd::Command as AssertCommand; // Renamed for clarity
use tempfile::tempdir;
use std::fs;
use std::path::Path;

// Helper function to create a git-switch command with isolated environment
fn get_git_switch_command(temp_home_path: &Path) -> Result<AssertCommand, Box<dyn std::error::Error>> {
    let mut cmd = AssertCommand::cargo_bin("git-switch")?;
    if cfg!(windows) {
        cmd.env("USERPROFILE", temp_home_path);
    } else {
        cmd.env("HOME", temp_home_path);
    }
    cmd.env_remove("GIT_CONFIG_GLOBAL");
    cmd.env_remove("GIT_CONFIG_SYSTEM");
    cmd.env_remove("GIT_CONFIG_NOSYSTEM"); // Ensure no system-wide git config interference
    Ok(cmd)
}

// Helper function to create a git command with isolated environment
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

fn setup_git_repo(repo_path: &Path, temp_home_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
        .args(&["remote", "add", "origin", "https://github.com/user/repo.git"])
        .current_dir(repo_path)
        .assert()
        .success();
    Ok(())
}

#[test]
fn test_add_account_and_list() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();
    let ssh_key_path = temp_home_path.join(".ssh/id_rsa_testacc");
    let ssh_key_path_str = ssh_key_path.to_str().unwrap();

    // Create dummy SSH key files for the test FIRST
    let ssh_dir = temp_home_path.join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    fs::File::create(&ssh_key_path)?;
    fs::File::create(ssh_key_path.with_extension("pub"))?;

    let mut cmd = get_git_switch_command(temp_home_path)?;

    // Add account
    cmd.args(&[
        "add",
        "testacc",
        "testuser",
        "testuser@example.com",
        "--ssh-key-path",
        ssh_key_path_str,
    ]);
    cmd.assert().success().stdout(predicate::str::contains("Account \'testacc\' added successfully!"));

    // List accounts
    let mut cmd_list = get_git_switch_command(temp_home_path)?;
    cmd_list.arg("list");
    cmd_list
        .assert()
        .success()
        .stdout(predicate::str::contains("testacc"))
        .stdout(predicate::str::contains("testuser"))
        .stdout(predicate::str::contains("testuser@example.com"))
        .stdout(predicate::str::contains(ssh_key_path.file_name().unwrap().to_str().unwrap())); // Check for file name

    Ok(())
}

#[test]
fn test_use_account_globally() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();
    
    let ssh_key_path_globalacc = temp_home_path.join(".ssh/id_rsa_globalacc");
    let ssh_key_path_globalacc_str = ssh_key_path_globalacc.to_str().unwrap();

    // Create dummy SSH key files FIRST
    let ssh_dir = temp_home_path.join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    fs::File::create(&ssh_key_path_globalacc)?;

    let mut cmd = get_git_switch_command(temp_home_path)?;

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
    let mut cmd_use = get_git_switch_command(temp_home_path)?;
    cmd_use.args(&["use", "globalacc"]);
    cmd_use.assert().success().stdout(predicate::str::contains("Global Git user.name and user.email set."));

    // Verify global git config (requires git command)
    let mut git_cmd = get_git_command(temp_home_path);
    git_cmd.args(&["config", "--global", "user.name"]);
    git_cmd.assert().success().stdout(predicate::str::contains("globaluser"));

    let mut git_cmd_email = get_git_command(temp_home_path);
    git_cmd_email.args(&["config", "--global", "user.email"]);
    git_cmd_email.assert().success().stdout(predicate::str::contains("global@example.com"));

    Ok(())
}

#[test]
fn test_account_subcommand_local_repo() -> Result<(), Box<dyn std::error::Error>> {
    let temp_config_dir = tempdir()?; 
    let temp_home_path = temp_config_dir.path();
    let repo_dir = tempdir()?; 

    setup_git_repo(repo_dir.path(), temp_home_path)?;

    let ssh_key_path_localacc = temp_home_path.join(".ssh/id_rsa_localacc");
    let ssh_key_path_localacc_str = ssh_key_path_localacc.to_str().unwrap();

    let ssh_dir_config = temp_home_path.join(".ssh"); 
    fs::create_dir_all(&ssh_dir_config)?;
    fs::File::create(&ssh_key_path_localacc)?;

    let mut cmd_add = get_git_switch_command(temp_home_path)?;
    cmd_add.args(&[
        "add",
        "localacc",
        "localuser",
        "local@example.com",
        "--ssh-key-path",
        ssh_key_path_localacc_str,
    ]).assert().success();

    // Use account for the local repository
    let mut cmd_account = get_git_switch_command(temp_home_path)?;
    cmd_account.current_dir(repo_dir.path()); 
    cmd_account.args(&["account", "localacc"]);
    cmd_account.assert().success().stdout(predicate::str::contains("Git user.name, user.email, and core.sshCommand set locally for this repository."));

    // Verify local git config
    let mut git_cmd_name = get_git_command(temp_home_path); // Git commands in repo dir still need isolated home for global fallback
    git_cmd_name.current_dir(repo_dir.path());
    git_cmd_name.args(&["config", "user.name"]);
    git_cmd_name.assert().success().stdout(predicate::str::contains("localuser"));

    let mut git_cmd_email = get_git_command(temp_home_path);
    git_cmd_email.current_dir(repo_dir.path());
    git_cmd_email.args(&["config", "user.email"]);
    git_cmd_email.assert().success().stdout(predicate::str::contains("local@example.com"));
    
    let mut git_cmd_ssh = get_git_command(temp_home_path);
    git_cmd_ssh.current_dir(repo_dir.path());
    git_cmd_ssh.args(&["config", "core.sshCommand"]);
    git_cmd_ssh.assert().success().stdout(predicate::str::contains("id_rsa_localacc"));

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
    cmd.assert().success().stdout(predicate::str::contains("Remote 'origin' URL updated to: git@github.com:user/repo.git"));

    // Verify remote URL
    let mut git_cmd = get_git_command(temp_home_path);
    git_cmd.current_dir(repo_dir.path());
    git_cmd.args(&["remote", "get-url", "origin"]);
    git_cmd.assert().success().stdout(predicate::str::contains("git@github.com:user/repo.git"));
    
    Ok(())
}

#[test]
fn test_remote_ssh_to_https() -> Result<(), Box<dyn std::error::Error>> {
    let temp_config_dir = tempdir()?;
    let temp_home_path = temp_config_dir.path();
    let repo_dir = tempdir()?;
    setup_git_repo(repo_dir.path(), temp_home_path)?;
    
    // Change remote to SSH first
    get_git_command(temp_home_path)
        .args(&["remote", "set-url", "origin", "git@github.com:user/another.git"])
        .current_dir(repo_dir.path())
        .assert()
        .success();

    let mut cmd = get_git_switch_command(temp_home_path)?;
    cmd.current_dir(repo_dir.path());
    cmd.args(&["remote", "--https"]);
    cmd.assert().success().stdout(predicate::str::contains("Remote 'origin' URL updated to: https://github.com/user/another.git"));

    // Verify remote URL
    let mut git_cmd = get_git_command(temp_home_path);
    git_cmd.current_dir(repo_dir.path());
    git_cmd.args(&["remote", "get-url", "origin"]);
    git_cmd.assert().success().stdout(predicate::str::contains("https://github.com/user/another.git"));

    Ok(())
}

#[test]
fn test_whoami_no_repo_global_set() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();

    let ssh_key_path_whoami_global = temp_home_path.join(".ssh/id_rsa_whoami_global");
    let ssh_key_path_whoami_global_str = ssh_key_path_whoami_global.to_str().unwrap();

    let ssh_dir = temp_home_path.join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    fs::File::create(&ssh_key_path_whoami_global)?;

    let mut cmd_add = get_git_switch_command(temp_home_path)?;
    cmd_add.args(&[
        "add",
        "globalacc", 
        "globaluser",
        "global@example.com",
        "--ssh-key-path",
        ssh_key_path_whoami_global_str, 
    ]).assert().success();
    
    let mut cmd_use = get_git_switch_command(temp_home_path)?;
    cmd_use.args(&["use", "globalacc"]).assert().success();

    let mut cmd_whoami = get_git_switch_command(temp_home_path)?;
    // Run whoami in a directory that is NOT a git repo, but uses the temp_home_path for config
    let non_repo_dir = tempdir()?; 
    cmd_whoami.current_dir(non_repo_dir.path()); 
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
    let temp_config_dir = tempdir()?; 
    let temp_home_path = temp_config_dir.path();
    let repo_dir = tempdir()?; 
    setup_git_repo(repo_dir.path(), temp_home_path)?;

    let ssh_key_path_localwhoami = temp_home_path.join(".ssh/id_rsa_localwhoami");
    let ssh_key_path_localwhoami_str = ssh_key_path_localwhoami.to_str().unwrap();

    let ssh_dir_config = temp_home_path.join(".ssh");
    fs::create_dir_all(&ssh_dir_config)?;
    fs::File::create(&ssh_key_path_localwhoami)?;

    let mut cmd_add = get_git_switch_command(temp_home_path)?;
    cmd_add.args(&[
        "add",
        "localwhoami",
        "localiam",
        "locali@example.com",
        "--ssh-key-path",
        ssh_key_path_localwhoami_str,
    ]).assert().success();

    // Set local git config using git-switch account command
    let mut cmd_account = get_git_switch_command(temp_home_path)?;
    cmd_account.current_dir(repo_dir.path());
    cmd_account.args(&["account", "localwhoami"]).assert().success();
    
    let mut cmd_whoami = get_git_switch_command(temp_home_path)?;
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
    let temp_home_path = temp_dir.path();
    let mut cmd_add = get_git_switch_command(temp_home_path)?;

    let mock_key_name = "test_auth_key";
    let mock_key_path = temp_home_path.join(".ssh").join(mock_key_name);
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

    let mut cmd_auth = get_git_switch_command(temp_home_path)?; 
    cmd_auth.args(&["auth", "test"]); 
    cmd_auth.write_stdin("1\\n");

    cmd_auth.assert()
        .success() 
        .stdout(predicate::str::contains(format!("testing with key: {}", mock_key_path.display()).as_str()))
        .stdout(predicate::str::contains("Attempting SSH authentication test..."))
        .stdout(predicate::str::contains("git@github.com"))
        .stdout(predicate::str::contains("git@gitlab.com"));
    Ok(())
}

#[test]
fn test_remove_account() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();
    
    let key_path_str = temp_home_path.join(".ssh/id_rsa_removeacc").to_str().unwrap().to_string();
    let pub_key_path_str = format!("{}.pub", key_path_str);

    let ssh_dir = temp_home_path.join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    fs::File::create(&key_path_str)?;
    fs::File::create(&pub_key_path_str)?;

    let mut cmd_add = get_git_switch_command(temp_home_path)?;
    cmd_add.args(&[
        "add",
        "removeacc",
        "removeuser",
        "remove@example.com",
        "--ssh-key-path",
        &key_path_str,
    ]);
    cmd_add.assert().success();

    let mut cmd_remove = get_git_switch_command(temp_home_path)?; 
    cmd_remove.args(&["remove", "removeacc"]);
    cmd_remove.write_stdin("y\\ny\\n"); 

    cmd_remove.assert()
        .success()
        .stdout(predicate::str::contains("Account 'removeacc' removed from configuration."))
        .stdout(predicate::str::contains("SSH private key file"))
        .stdout(predicate::str::contains("deleted"))
        .stdout(predicate::str::contains("SSH public key file"));

    assert!(!Path::new(&key_path_str).exists());
    assert!(!Path::new(&pub_key_path_str).exists());

    let mut cmd_list = get_git_switch_command(temp_home_path)?;
    cmd_list.arg("list");
    cmd_list.assert().success().stdout(predicate::str::contains("No saved accounts.").or(predicate::str::contains("Saved Git Accounts:").not()));

    Ok(())
}

#[test]
fn test_add_account_default_ssh_key_generation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let temp_home_path = temp_dir.path();
    let mut cmd = get_git_switch_command(temp_home_path)?;

    // Add account without specifying SSH key path
    cmd.args(&["add", "defaultkeyacc", "defaultkeyuser", "defaultkey@example.com"]);
    
    // This command might take a moment if ssh-keygen is slow.
    // Consider adding a timeout to the assert if tests become flaky.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account 'defaultkeyacc' added successfully!"))
        .stdout(predicate::str::contains("Generating SSH key at"))
        .stdout(predicate::str::contains("id_rsa_defaultkeyacc")); 

    let expected_key_path = temp_home_path.join(".ssh/id_rsa_defaultkeyacc");
    let expected_pub_key_path = temp_home_path.join(".ssh/id_rsa_defaultkeyacc.pub");
    
    assert!(expected_key_path.exists(), "Private key file should be generated by the application at {}", expected_key_path.display());
    assert!(expected_pub_key_path.exists(), "Public key file should be generated by the application at {}", expected_pub_key_path.display());

    let ssh_config_path = temp_home_path.join(".ssh/config");
    assert!(ssh_config_path.exists());
    let ssh_config_content = fs::read_to_string(ssh_config_path)?;
    assert!(ssh_config_content.contains("Host github.com-defaultkeyacc"));
    assert!(ssh_config_content.contains("IdentityFile"));
    assert!(ssh_config_content.contains("id_rsa_defaultkeyacc"));

    Ok(())
}