use crate::config::{Account, Config};
use crate::error::{GitSwitchError, Result};
use crate::git;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents a discovered Git repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredRepo {
    pub path: PathBuf,
    pub remote_url: Option<String>,
    pub current_user_name: Option<String>,
    pub current_user_email: Option<String>,
    pub suggested_account: Option<String>,
    pub account_confidence: f32, // 0.0 to 1.0
    pub last_commit_author: Option<String>,
    pub branch: Option<String>,
}

/// Repository discovery and bulk operations manager
pub struct RepoManager {
    config: Config,
    discovered_repos: Vec<DiscoveredRepo>,
}

impl RepoManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            discovered_repos: Vec::new(),
        }
    }

    /// Discover Git repositories recursively from a given path
    pub fn discover_repositories(
        &mut self,
        search_path: &Path,
        max_depth: Option<usize>,
    ) -> Result<()> {
        println!(
            "{} Discovering Git repositories in {}...",
            "🔍".cyan(),
            search_path.display()
        );

        let repos = self.find_git_repositories(search_path, max_depth.unwrap_or(5))?;

        if repos.is_empty() {
            println!(
                "{} No Git repositories found in {}",
                "ℹ".blue(),
                search_path.display()
            );
            return Ok(());
        }

        println!(
            "{} Found {} repositories. Analyzing...",
            "✓".green(),
            repos.len()
        );

        // Create progress bar
        let pb = ProgressBar::new(repos.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        self.discovered_repos.clear();

        for repo_path in repos {
            let discovered = self.analyze_repository(&repo_path)?;
            self.discovered_repos.push(discovered);
            pb.inc(1);
        }

        pb.finish_with_message("Analysis complete!");

        println!(
            "{} Analyzed {} repositories",
            "✓".green(),
            self.discovered_repos.len()
        );
        self.print_discovery_summary()?;

        Ok(())
    }

    fn find_git_repositories(&self, path: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
        let mut repositories = Vec::new();
        Self::find_git_repositories_recursive(path, max_depth, 0, &mut repositories)?;
        Ok(repositories)
    }

    fn find_git_repositories_recursive(
        path: &Path,
        max_depth: usize,
        current_depth: usize,
        repositories: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }

        // Check if current directory is a Git repository
        if path.join(".git").exists() {
            repositories.push(path.to_path_buf());
            // Don't recurse into subdirectories of Git repositories
            return Ok(());
        }

        // Recurse into subdirectories
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir()
                    && !entry_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with('.')
                {
                    Self::find_git_repositories_recursive(
                        &entry_path,
                        max_depth,
                        current_depth + 1,
                        repositories,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn analyze_repository(&self, repo_path: &Path) -> Result<DiscoveredRepo> {
        let original_dir = std::env::current_dir().map_err(GitSwitchError::Io)?;

        // Change to repository directory
        std::env::set_current_dir(repo_path).map_err(GitSwitchError::Io)?;

        let result = self.analyze_current_repository(repo_path);

        // Restore original directory
        std::env::set_current_dir(original_dir).map_err(GitSwitchError::Io)?;

        result
    }

    fn analyze_current_repository(&self, repo_path: &Path) -> Result<DiscoveredRepo> {
        let remote_url = git::get_remote_url("origin").ok();
        let current_user_name = git::get_local_config_key("user.name").ok();
        let current_user_email = git::get_local_config_key("user.email").ok();
        let branch = git::get_current_branch().ok();

        // Get last commit author
        let last_commit_author = std::process::Command::new("git")
            .args(["log", "-1", "--pretty=format:%an <%ae>"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            });

        // Detect suggested account
        let (suggested_account, confidence) = if let Some(url) = &remote_url {
            match crate::detection::detect_account_for_remote_url(&self.config, url) {
                Ok(Some(account)) => (Some(account), 0.9),
                _ => {
                    // Try to match by email or name
                    self.find_matching_account_by_user(&current_user_email, &current_user_name)
                }
            }
        } else {
            self.find_matching_account_by_user(&current_user_email, &current_user_name)
        };

        Ok(DiscoveredRepo {
            path: repo_path.to_path_buf(),
            remote_url,
            current_user_name,
            current_user_email,
            suggested_account,
            account_confidence: confidence,
            last_commit_author,
            branch,
        })
    }

    fn find_matching_account_by_user(
        &self,
        email: &Option<String>,
        name: &Option<String>,
    ) -> (Option<String>, f32) {
        let mut best_match = None;
        let mut best_confidence = 0.0;

        for (account_name, account) in &self.config.accounts {
            let mut confidence = 0.0;
            let mut matches = 0;
            let mut total_checks = 0;

            // Check email match
            if let (Some(repo_email), account_email) = (email, &account.email) {
                total_checks += 1;
                if repo_email == account_email {
                    matches += 1;
                    confidence += 0.6; // Email is highly indicative
                }
            }

            // Check name match
            if let (Some(repo_name), account_name_val) = (name, &account.name) {
                total_checks += 1;
                if repo_name == account_name_val {
                    matches += 1;
                    confidence += 0.4; // Name is less reliable than email
                }
            }

            if total_checks > 0 {
                confidence *= matches as f32 / total_checks as f32;

                if confidence > best_confidence {
                    best_confidence = confidence;
                    best_match = Some(account_name.clone());
                }
            }
        }

        (best_match, best_confidence)
    }

    fn print_discovery_summary(&self) -> Result<()> {
        let mut with_suggestions = 0;
        let mut high_confidence = 0;
        let mut mismatched = 0;

        for repo in &self.discovered_repos {
            if repo.suggested_account.is_some() {
                with_suggestions += 1;
                if repo.account_confidence > 0.7 {
                    high_confidence += 1;
                }
            }

            // Check for potential mismatches
            if let (Some(suggested), Some(current_email)) =
                (&repo.suggested_account, &repo.current_user_email)
            {
                if let Some(account) = self.config.accounts.get(suggested) {
                    if current_email != &account.email {
                        mismatched += 1;
                    }
                }
            }
        }

        println!();
        println!("{}", "Discovery Summary:".bold().underline());
        println!(
            "  Total repositories: {}",
            self.discovered_repos.len().to_string().cyan()
        );
        println!(
            "  With account suggestions: {}",
            with_suggestions.to_string().green()
        );
        println!(
            "  High confidence matches: {}",
            high_confidence.to_string().yellow()
        );
        if mismatched > 0 {
            println!("  Potential mismatches: {}", mismatched.to_string().red());
        }
        println!();

        Ok(())
    }

    /// List discovered repositories with details
    pub fn list_discovered(&self) -> Result<()> {
        if self.discovered_repos.is_empty() {
            println!(
                "{} No repositories discovered yet. Run discovery first.",
                "ℹ".blue()
            );
            return Ok(());
        }

        println!("{}", "Discovered Repositories:".bold().underline());
        println!();

        for (i, repo) in self.discovered_repos.iter().enumerate() {
            println!(
                "{} {}",
                format!("{}.", i + 1).cyan(),
                repo.path.display().to_string().bold()
            );

            if let Some(url) = &repo.remote_url {
                println!("   Remote: {}", url.dimmed());
            }

            if let Some(branch) = &repo.branch {
                println!("   Branch: {}", branch.cyan());
            }

            // Current configuration
            match (&repo.current_user_name, &repo.current_user_email) {
                (Some(name), Some(email)) => {
                    println!("   Current: {} <{}>", name, email);
                }
                (Some(name), None) => {
                    println!("   Current: {}", name);
                }
                (None, Some(email)) => {
                    println!("   Current: <{}>", email);
                }
                (None, None) => {
                    println!("   Current: {}", "Not configured".red());
                }
            }

            // Suggested account
            if let Some(suggested) = &repo.suggested_account {
                let confidence_color = if repo.account_confidence > 0.7 {
                    suggested.green()
                } else if repo.account_confidence > 0.4 {
                    suggested.yellow()
                } else {
                    suggested.normal()
                };

                println!(
                    "   Suggested: {} ({}% confidence)",
                    confidence_color,
                    (repo.account_confidence * 100.0) as u8
                );
            } else {
                println!("   Suggested: {}", "None".dimmed());
            }

            println!();
        }

        Ok(())
    }

    /// Apply account configurations to multiple repositories
    pub fn bulk_apply(&mut self, dry_run: bool, force: bool) -> Result<()> {
        if self.discovered_repos.is_empty() {
            return Err(GitSwitchError::NoRepositoriesDiscovered);
        }

        let applicable_repos: Vec<_> = self
            .discovered_repos
            .iter()
            .filter(|repo| repo.suggested_account.is_some())
            .collect();

        if applicable_repos.is_empty() {
            println!(
                "{} No repositories with account suggestions found",
                "ℹ".blue()
            );
            return Ok(());
        }

        println!("{} repositories with suggestions:", applicable_repos.len());

        if dry_run {
            println!("{}", "DRY RUN - No changes will be made".yellow().bold());
        }

        println!();

        for repo in &applicable_repos {
            let suggested_account = repo.suggested_account.as_ref().unwrap();
            let account = self.config.accounts.get(suggested_account).unwrap();

            println!("{} {}", "▶".green(), repo.path.display());
            println!("  Account: {}", suggested_account.cyan());

            println!("  Name: {}", account.name);
            println!("  Email: {}", account.email);

            if !dry_run {
                if !force && repo.account_confidence < 0.5 {
                    println!(
                        "  {}: Low confidence, skipping (use --force to apply)",
                        "⚠".yellow()
                    );
                    continue;
                }

                // Apply the account configuration
                match self.apply_account_to_repo(&repo.path, suggested_account) {
                    Ok(_) => println!("  {}: Applied successfully", "✓".green()),
                    Err(e) => println!("  {}: Failed - {}", "✗".red(), e),
                }
            }

            println!();
        }

        if dry_run {
            println!("Run without --dry-run to apply changes");
        } else {
            println!("{} Bulk operation completed", "✓".green());
        }

        Ok(())
    }

    fn apply_account_to_repo(&self, repo_path: &Path, account_name: &str) -> Result<()> {
        let account = self.config.accounts.get(account_name).ok_or_else(|| {
            GitSwitchError::AccountNotFound {
                name: account_name.to_string(),
            }
        })?;

        let original_dir = std::env::current_dir().map_err(GitSwitchError::Io)?;

        // Change to repository directory
        std::env::set_current_dir(repo_path).map_err(GitSwitchError::Io)?;

        let result = self.apply_account_config(account);

        // Restore original directory
        std::env::set_current_dir(original_dir).map_err(GitSwitchError::Io)?;

        result
    }

    fn apply_account_config(&self, account: &Account) -> Result<()> {
        // Set user name
        git::set_local_config_key("user.name", &account.name)?;

        // Set user email
        git::set_local_config_key("user.email", &account.email)?;

        // Set SSH key if available
        if !account.ssh_key_path.is_empty() {
            git::set_local_config_key(
                "core.sshCommand",
                &format!("ssh -i {}", account.ssh_key_path),
            )?;
        }

        Ok(())
    }

    /// Generate a report of repository analysis
    pub fn generate_report(&self, output_path: Option<&Path>) -> Result<()> {
        let report = self.create_report()?;

        match output_path {
            Some(path) => {
                std::fs::write(path, &report).map_err(GitSwitchError::Io)?;
                println!("{} Report saved to {}", "✓".green(), path.display());
            }
            None => {
                println!("{}", report);
            }
        }

        Ok(())
    }

    fn create_report(&self) -> Result<String> {
        let mut report = String::new();

        report.push_str("# Git Repository Analysis Report\n");
        report.push_str(&format!(
            "Generated: {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ));

        report.push_str("## Summary\n");
        report.push_str(&format!(
            "- Total repositories: {}\n",
            self.discovered_repos.len()
        ));

        let with_suggestions = self
            .discovered_repos
            .iter()
            .filter(|r| r.suggested_account.is_some())
            .count();
        report.push_str(&format!("- With suggestions: {}\n", with_suggestions));

        let high_confidence = self
            .discovered_repos
            .iter()
            .filter(|r| r.account_confidence > 0.7)
            .count();
        report.push_str(&format!("- High confidence: {}\n\n", high_confidence));

        report.push_str("## Repository Details\n\n");

        for (i, repo) in self.discovered_repos.iter().enumerate() {
            report.push_str(&format!("### {}. {}\n", i + 1, repo.path.display()));

            if let Some(url) = &repo.remote_url {
                report.push_str(&format!("- **Remote**: {}\n", url));
            }

            if let Some(branch) = &repo.branch {
                report.push_str(&format!("- **Branch**: {}\n", branch));
            }

            match (&repo.current_user_name, &repo.current_user_email) {
                (Some(name), Some(email)) => {
                    report.push_str(&format!("- **Current Config**: {} <{}>\n", name, email));
                }
                (Some(name), None) => {
                    report.push_str(&format!("- **Current Config**: {}\n", name));
                }
                (None, Some(email)) => {
                    report.push_str(&format!("- **Current Config**: <{}>\n", email));
                }
                (None, None) => {
                    report.push_str("- **Current Config**: Not configured\n");
                }
            }

            if let Some(suggested) = &repo.suggested_account {
                report.push_str(&format!(
                    "- **Suggested Account**: {} ({}% confidence)\n",
                    suggested,
                    (repo.account_confidence * 100.0) as u8
                ));
            }

            report.push('\n');
        }

        Ok(report)
    }

    /// Interactive repository selection and configuration
    pub fn interactive_configure(&mut self) -> Result<()> {
        use dialoguer::{Confirm, MultiSelect};

        if self.discovered_repos.is_empty() {
            return Err(GitSwitchError::NoRepositoriesDiscovered);
        }

        let repos_with_suggestions: Vec<_> = self
            .discovered_repos
            .iter()
            .enumerate()
            .filter(|(_, repo)| repo.suggested_account.is_some())
            .collect();

        if repos_with_suggestions.is_empty() {
            println!(
                "{} No repositories with account suggestions found",
                "ℹ".blue()
            );
            return Ok(());
        }

        let items: Vec<String> = repos_with_suggestions
            .iter()
            .map(|(_, repo)| {
                format!(
                    "{} -> {}",
                    repo.path.display(),
                    repo.suggested_account.as_ref().unwrap()
                )
            })
            .collect();

        let selections = MultiSelect::new()
            .with_prompt("Select repositories to configure")
            .items(&items)
            .interact()?;

        if selections.is_empty() {
            println!("No repositories selected");
            return Ok(());
        }

        println!("\nYou selected {} repositories:", selections.len());
        for &idx in &selections {
            let (_, repo) = repos_with_suggestions[idx];
            println!(
                "  {} -> {}",
                repo.path.display(),
                repo.suggested_account.as_ref().unwrap()
            );
        }

        let confirm = Confirm::new()
            .with_prompt("Apply these configurations?")
            .interact()?;

        if !confirm {
            println!("Operation cancelled");
            return Ok(());
        }

        // Apply configurations
        for &idx in &selections {
            let (_, repo) = repos_with_suggestions[idx];
            let account_name = repo.suggested_account.as_ref().unwrap();

            match self.apply_account_to_repo(&repo.path, account_name) {
                Ok(_) => println!(
                    "{} {} -> {}",
                    "✓".green(),
                    repo.path.display(),
                    account_name
                ),
                Err(e) => println!(
                    "{} {} -> {} ({})",
                    "✗".red(),
                    repo.path.display(),
                    account_name,
                    e
                ),
            }
        }

        println!("{} Interactive configuration completed", "✓".green());
        Ok(())
    }
}
