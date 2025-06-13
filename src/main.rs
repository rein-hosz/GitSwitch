mod commands;
mod config;
mod error;
mod ssh;
mod git;
mod utils;
mod backup;
mod validation;
mod detection;
mod templates;
mod analytics;
mod profiles;
mod repository;
mod completions;
mod manpages;

use clap::{Parser, Subcommand, CommandFactory};
use crate::error::Result;
use crate::backup::ExportFormat;
use std::path::PathBuf;
use crate::error::GitSwitchError;
use std::process::exit;
use colored::*;

/// Represents the command-line interface for git-switch.
#[derive(Parser, Debug)]
#[clap(name = "git-switch",
    author,
    version = env!("APP_LONG_VERSION"), // This will be used by -V
    long_version = env!("APP_VERSION"),  // This will be used by --version
    about,
    long_about = None
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// Enable verbose logging
    #[clap(long, short, global = true)]
    verbose: bool,
    /// Disable colored output
    #[clap(long, global = true)]
    no_color: bool,
}

/// Defines the available subcommands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Adds a new Git account
    Add {
        /// Name of the account (e.g., "personal", "work")
        name: String,
        /// Username for Git config (e.g., "John Doe")
        username: String,
        /// Email for Git config (e.g., "john.doe@example.com")
        email: String,
        /// Optional path to the SSH key for this account
        #[clap(long)]
        ssh_key_path: Option<PathBuf>,
        /// Use interactive mode for account creation
        #[clap(long, short)]
        interactive: bool,
        /// Provider preset (github, gitlab, bitbucket)
        #[clap(long)]
        provider: Option<String>,
    },
    /// Lists all configured Git accounts
    List {
        /// Show detailed information
        #[clap(long, short)]
        detailed: bool,
    },
    /// Switches to a specified Git account for the current repository
    Use {
        /// Name of the account to use
        name: String,
    },
    /// Removes a configured Git account
    Remove {
        /// Name of the account to remove
        name: String,
        /// Skip confirmation prompt
        #[clap(long, short = 'y', action)]
        no_prompt: bool,
    },
    /// Manages account settings for the current repository (applies account to current repo)
    Account {
        /// Name of the account to apply to the current repository
        name: String,
    },
    /// Modifies the remote URL protocol for the current repository
    Remote {
        /// Switch remote to HTTPS
        #[clap(long, conflicts_with = "ssh")]
        https: bool,
        /// Switch remote to SSH
        #[clap(long, conflicts_with = "https")]
        ssh: bool,
    },
    /// Shows the current Git identity and remote status
    Whoami,
    /// Authentication related commands
    Auth(AuthOpts),
    /// Backup and restore commands
    Backup(BackupOpts),
    /// Profile management commands
    Profile(ProfileOpts),
    /// Template management commands
    Template(TemplateOpts),
    /// Analytics and usage statistics
    Analytics(AnalyticsOpts),
    /// Repository detection and suggestions
    Detect,
    /// Repository discovery and bulk operations
    Repo(RepoOpts),
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[clap(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Generate man pages
    Man {
        /// Output directory for man pages
        #[clap(long, short)]
        output_dir: Option<String>,
    },
}

#[derive(Parser, Debug)]
struct AuthOpts {
    #[clap(subcommand)]
    command: AuthCommands,
}

#[derive(Subcommand, Debug)]
enum AuthCommands {
    /// Tests SSH authentication for the currently configured account or a specific key
    Test,
}

#[derive(Parser, Debug)]
struct BackupOpts {
    #[clap(subcommand)]
    command: BackupCommands,
}

#[derive(Subcommand, Debug)]
enum BackupCommands {
    /// Create a backup of the current configuration
    Create {
        /// Path to save the backup file
        #[clap(long, short)]
        output: Option<PathBuf>,
    },
    /// Restore configuration from a backup file
    Restore {
        /// Path to the backup file
        backup_file: PathBuf,
    },
    /// Export accounts to a file
    Export {
        /// Output file path
        output: PathBuf,
        /// Export format (toml, json)
        #[clap(long, short, default_value = "toml")]
        format: ExportFormat,
    },
    /// Import accounts from a file
    Import {
        /// Input file path
        input: PathBuf,
        /// Merge with existing accounts instead of replacing
        #[clap(long, short)]
        merge: bool,
    },
}

#[derive(Parser, Debug)]
struct ProfileOpts {
    #[clap(subcommand)]
    command: ProfileCommands,
}

#[derive(Subcommand, Debug)]
enum ProfileCommands {
    /// Create a new profile
    Create {
        /// Profile name
        name: String,
        /// Account names to include in this profile (comma-separated)
        #[clap(long, short, value_delimiter = ',')]
        accounts: Vec<String>,
        /// Description for the profile
        #[clap(long, short)]
        description: Option<String>,
        /// Default account for this profile
        #[clap(long)]
        default: Option<String>,
    },
    /// List all profiles
    List,
    /// Switch to a profile
    Use {
        /// Profile name
        name: String,
        /// Override the default account
        #[clap(long, short)]
        account: Option<String>,
    },
    /// Update an existing profile
    Update {
        /// Profile name
        name: String,
        /// Description for the profile
        #[clap(long, short)]
        description: Option<String>,
        /// Add accounts to the profile (comma-separated)
        #[clap(long, value_delimiter = ',')]
        add_accounts: Vec<String>,
        /// Remove accounts from the profile (comma-separated)
        #[clap(long, value_delimiter = ',')]
        remove_accounts: Vec<String>,
        /// Set default account for this profile
        #[clap(long)]
        default: Option<String>,
    },
    /// Remove a profile
    Remove {
        /// Profile name
        name: String,
    },
    /// Show profile statistics
    Stats,
}

#[derive(Parser, Debug)]
struct TemplateOpts {
    #[clap(subcommand)]
    command: TemplateCommands,
}

#[derive(Subcommand, Debug)]
enum TemplateCommands {
    /// List available account templates
    List,
    /// Create account from template
    Use {
        /// Template name (github, gitlab, bitbucket, etc.)
        template: String,
        /// Account name
        name: String,
        /// Username
        username: String,
        /// Email address
        email: String,
    },
}

#[derive(Parser, Debug)]
struct AnalyticsOpts {
    #[clap(subcommand)]
    command: AnalyticsCommands,
}

#[derive(Subcommand, Debug)]
enum AnalyticsCommands {
    /// Show usage analytics
    Show,
    /// Clear analytics data
    Clear,
}

#[derive(Parser, Debug)]
struct RepoOpts {
    #[clap(subcommand)]
    command: RepoCommands,
}

#[derive(Subcommand, Debug)]
enum RepoCommands {
    /// Discover Git repositories in a directory
    Discover {
        /// Path to search for repositories
        #[clap(default_value = ".")]
        path: std::path::PathBuf,
        /// Maximum depth to search
        #[clap(long, short, default_value_t = 5)]
        max_depth: usize,
    },
    /// List discovered repositories
    List,
    /// Apply account configurations to repositories
    Apply {
        /// Perform a dry run without making changes
        #[clap(long)]
        dry_run: bool,
        /// Force application even for low-confidence matches
        #[clap(long)]
        force: bool,
    },
    /// Generate a report of repository analysis
    Report {
        /// Output path for the report
        #[clap(long, short)]
        output: Option<std::path::PathBuf>,
    },
    /// Interactive repository configuration
    Interactive,
}

/// Main function to run the git-switch application.
fn main() {
    if let Err(e) = run_cli() {
        let error_msg = if std::env::var("NO_COLOR").is_ok() {
            format!("Error: {}", e)
        } else {
            format!("{}: {}", "Error".red().bold(), e)
        };
        eprintln!("{}", error_msg);

        // Attempt to downcast anyhow::Error to GitSwitchError
        if let Some(git_switch_error) = e.downcast_ref::<GitSwitchError>() {
            exit(git_switch_error.exit_code());
        } else {
            // If it's not a GitSwitchError, exit with a generic code
            exit(1);
        }
    }
}

/// Helper function to contain the main CLI logic.
fn run_cli() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt::init();
    }
    
    // Set color preference
    if cli.no_color {
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
    }
    
    // Perform startup validation
    if let Err(e) = validation::validate_startup() {
        tracing::warn!("Startup validation failed: {}", e);
    }
    
    let mut config = config::load_config()?;

    match cli.command {
        Commands::Add { name, username, email, ssh_key_path, interactive, provider } => {
            if interactive {
                commands::add_account_interactive(&mut config, &name)?;
            } else {
                commands::add_account(&mut config, &name, &username, &email, ssh_key_path, provider)?;
            }
        }
        Commands::List { detailed } => commands::list_accounts(&config, detailed)?,
        Commands::Use { name } => commands::use_account_globally(&config, &name)?,
        Commands::Remove { name, no_prompt } => {
            commands::remove_account(&mut config, &name, no_prompt)?;
        }
        Commands::Account { name } => {
            commands::handle_account_subcommand(&config, &name)?;
        }
        Commands::Remote { https, ssh } => {
            commands::handle_remote_subcommand(https, ssh)?;
        }
        Commands::Whoami => {
            commands::handle_whoami_subcommand(&config)?;
        }
        Commands::Auth(auth_opts) => match auth_opts.command {
            AuthCommands::Test => {
                commands::handle_auth_test_subcommand(&config)?;
            }
        },
        Commands::Backup(backup_opts) => match backup_opts.command {
            BackupCommands::Create { output } => {
                backup::backup_config(output.as_deref())?;
            }
            BackupCommands::Restore { backup_file } => {
                backup::restore_config(&backup_file)?;
            }
            BackupCommands::Export { output, format } => {
                backup::export_accounts(&output, format)?;
            }
            BackupCommands::Import { input, merge } => {
                backup::import_accounts(&input, merge)?;
            }
        },
        Commands::Profile(profile_opts) => match profile_opts.command {
            ProfileCommands::Create { name, accounts, description, default } => {
                let mut profile_manager = profiles::ProfileManager::new(config.clone())?;
                profile_manager.create_profile(name, description, accounts, default)?;
            }
            ProfileCommands::List => {
                let profile_manager = profiles::ProfileManager::new(config)?;
                profile_manager.list_profiles()?;
            }
            ProfileCommands::Use { name, account } => {
                let mut profile_manager = profiles::ProfileManager::new(config)?;
                profile_manager.switch_profile(&name, account)?;
            }
            ProfileCommands::Update { name, description, add_accounts, remove_accounts, default } => {
                let mut profile_manager = profiles::ProfileManager::new(config)?;
                profile_manager.update_profile(&name, description, add_accounts, remove_accounts, default)?;
            }
            ProfileCommands::Remove { name } => {
                let mut profile_manager = profiles::ProfileManager::new(config)?;
                profile_manager.delete_profile(&name)?;
            }
            ProfileCommands::Stats => {
                let profile_manager = profiles::ProfileManager::new(config)?;
                profile_manager.get_profile_stats()?;
            }
        },
        Commands::Template(template_opts) => match template_opts.command {
            TemplateCommands::List => {
                templates::list_templates();
            }
            TemplateCommands::Use { template, name, username, email } => {
                let tmpl = templates::get_template(&template)?;
                let account = templates::create_account_from_template(&name, &username, &email, &tmpl);
                config.accounts.insert(name.clone(), account);
                config::save_config(&config)?;
                println!("{} Account '{}' created from {} template", "✓".green().bold(), name.cyan(), template.cyan());
            }
        },
        Commands::Analytics(analytics_opts) => match analytics_opts.command {
            AnalyticsCommands::Show => {
                analytics::show_analytics(&config)?;
            }
            AnalyticsCommands::Clear => {
                analytics::clear_analytics()?;
            }
        },
        Commands::Detect => {
            detection::suggest_account(&config)?;
            detection::check_account_mismatch(&config)?;
        },
        Commands::Repo(repo_opts) => {
            let mut repo_manager = repository::RepoManager::new(config);
            match repo_opts.command {
                RepoCommands::Discover { path, max_depth } => {
                    repo_manager.discover_repositories(&path, Some(max_depth))?;
                }
                RepoCommands::List => {
                    repo_manager.list_discovered()?;
                }
                RepoCommands::Apply { dry_run, force } => {
                    repo_manager.bulk_apply(dry_run, force)?;
                }
                RepoCommands::Report { output } => {
                    repo_manager.generate_report(output.as_deref())?;
                }
                RepoCommands::Interactive => {
                    repo_manager.interactive_configure()?;
                }
            }
        },
        Commands::Completions { shell } => {
            completions::generate_completions(shell, &mut Cli::command());
            completions::print_installation_instructions(shell);
        }
        Commands::Man { output_dir } => {
            if let Some(dir) = output_dir {
                if let Err(e) = manpages::generate_all_man_pages(&Cli::command(), Some(&dir)) {
                    eprintln!("Error generating man pages: {}", e);
                    exit(1);
                }
            } else {
                if let Err(e) = manpages::generate_man_page(&Cli::command()) {
                    eprintln!("Error generating man page: {}", e);
                    exit(1);
                }
            }
            manpages::print_man_installation_instructions();
        },
    }
    Ok(())
}
