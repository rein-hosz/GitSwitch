use std::process::Command;
use crate::utils::run_command;

pub fn update_git_remote(username: &str, repo_url: &str) {
    let repo_name = if repo_url.contains('/') {
        // Handle full repo path like "username/repo"
        repo_url.split('/').last().unwrap_or("").replace(".git", "")
    } else {
        // Handle just repo name
        repo_url.replace(".git", "")
    };

    // Create remote URL using the host alias from SSH config
    let remote_url = format!("git@github.com:{}/{}.git", username, repo_name);

    println!("🔄 Updating Git remote URL to: {}", remote_url);

    // Check if origin remote exists
    let output = Command::new("git")
        .args(["remote"])
        .output()
        .expect("Failed to execute git remote command");

    let remotes = String::from_utf8_lossy(&output.stdout);

    if remotes.lines().any(|line| line == "origin") {
        println!("Removing existing 'origin' remote...");
        run_command("git", &["remote", "remove", "origin"]);
    }

    println!("Adding new 'origin' remote...");
    run_command("git", &["remote", "add", "origin", &remote_url]);

    println!("✅ Git remote URL updated successfully!");
}
