use crate::error::{Result};
use crate::utils::{run_command, run_command_with_output};

pub fn update_git_remote(account_name: &str, repo_name_input: &str) -> Result<()> {
    // Construct the host alias used in SSH config
    let host_alias = format!("github-{}", account_name.replace(" ", "_").to_lowercase());

    // The repo_name_input could be 'repo' or 'owner/repo'.
    // The SSH URL format is git@HOST_ALIAS:OWNER/REPO.git
    // If only 'repo' is given, we need a way to infer owner, or this needs to be 'owner/repo'.
    // For now, let's assume repo_name_input is 'owner/repo'.
    // If it's just 'repo', this will likely fail unless the SSH config handles it, which is not standard.
    // A more robust solution might involve asking for owner if not provided, or using current git config.

    let remote_url = if repo_name_input.contains('/') {
        format!("git@{}:{}.git", host_alias, repo_name_input)
    } else {
        // This case is ambiguous. Standard git clone needs owner/repo.
        // Let's assume the username of the account IS the owner for this simplified case.
        // This is a common convention but not always true.
        format!("git@{}:{}/{}.git", host_alias, account_name, repo_name_input)
    };

    println!("🔄 Attempting to update Git remote URL to: {}", remote_url);

    // Check if 'origin' remote exists
    let output = run_command_with_output("git", &["remote"], None)?;
    let remotes = String::from_utf8_lossy(&output.stdout);

    if remotes.lines().any(|line| line.trim() == "origin") {
        println!("Removing existing \'origin\' remote...");
        run_command("git", &["remote", "remove", "origin"], None)?;
    }

    println!("Adding new \'origin\' remote with URL: {}...", remote_url);
    run_command("git", &["remote", "add", "origin", &remote_url], None)?;

    println!("✅ Git remote \'origin\' updated successfully to use account \'{}\'!", account_name);
    Ok(())
}
