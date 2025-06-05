use clap::{Arg, Command};
use crate::commands::{add_account, list_accounts, use_account, remove_account};
use crate::commands::{add_account, list_accounts, use_account};

mod commands;
mod config;
mod ssh;
mod git;
mod utils;

fn main() {
    let matches = Command::new("git-switch")
        .version("1.0")
        .about("CLI tool to switch between multiple Git accounts")
        .subcommand(
            Command::new("add")
                .about("Add a new Git account")
                .arg(Arg::new("name").required(true).help("Name for the account (e.g. 'Work', 'Personal')"))
                .arg(Arg::new("username").required(true).help("Git username"))
                .arg(Arg::new("email").required(true).help("Git email address")),
            )
        .subcommand(
            Command::new("use")
                .about("Switch to a saved Git account")
                .arg(Arg::new("name").required(true).help("Name or username of the account to use")),
            )
        .subcommand(Command::new("list").about("List all saved Git accounts"))
        .subcommand(
            Command::new("remove")
                .about("Remove a saved Git account and its SSH key")
                .arg(Arg::new("name").required(true).help("Name of the account to remove")),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("add", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            let username = sub_m.get_one::<String>("username").unwrap();
            let email = sub_m.get_one::<String>("email").unwrap();
            add_account(name, username, email);
        }
        Some(("use", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            use_account(name);
        }
        Some(("list", _)) => {
            list_accounts();
        }
        Some(("remove", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            remove_account(name);
        }
        _ => {
            list_accounts();
        }
        _ => {
            println!("Use 'git-switch --help' to see available commands.");
        }
    }
}
