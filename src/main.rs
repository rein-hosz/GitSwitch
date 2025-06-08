mod commands;
mod config;
mod error;
mod ssh;
mod git;
mod utils;

use clap::{Parser, Subcommand};
use crate::error::Result;
use std::path::PathBuf;

/// Represents the command-line interface for git-switch.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
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
    },
    /// Lists all configured Git accounts
    List,
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

/// Main function to run the git-switch application.
fn main() {
    if let Err(e) = run() {
        eprintln!("❌ Error: {}", e);
        std::process::exit(e.exit_code());
    }
}

fn run() -> Result<()> {
    let cli = Cli::try_parse().map_err(error::GitSwitchError::Clap)?;
    let mut config = config::load_config()?;

    match cli.command {
        Commands::Add { name, username, email, ssh_key_path } => {
            commands::add_account(&mut config, &name, &username, &email, ssh_key_path)
        }
        Commands::List => commands::list_accounts(&config),
        Commands::Use { name } => commands::use_account_globally(&config, &name),
        Commands::Remove { name, no_prompt } => {
            commands::remove_account(&mut config, &name, no_prompt)
        }
        Commands::Account { name } => {
            commands::handle_account_subcommand(&config, &name)
        }
        Commands::Remote { https, ssh } => {
            commands::handle_remote_subcommand(https, ssh)
        }
        Commands::Whoami => {
            commands::handle_whoami_subcommand(&config)
        }
        Commands::Auth(auth_opts) => match auth_opts.command {
            AuthCommands::Test => {
                commands::handle_auth_test_subcommand(&config)
            }
        },
    }
}
