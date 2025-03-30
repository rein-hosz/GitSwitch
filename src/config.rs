use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use crate::utils::file_exists;

pub fn get_config_path() -> String {
    shellexpand::tilde("~/.git-switch-accounts").to_string()
}

#[derive(Debug, Clone)]
pub struct Account {
    pub name: String,
    pub username: String,
    pub email: String,
    pub ssh_key: String,
}

pub fn save_account(account: &Account) {
    let config_path = get_config_path();
    let entry = format!("{}|{}|{}|{}\n", account.name, account.username, account.email, account.ssh_key);
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&config_path)
        .expect("Failed to open config file");
    file.write_all(entry.as_bytes()).expect("Failed to save account");
    println!("✅ Account '{}' saved.", account.name);
}

pub fn load_accounts() -> Vec<Account> {
    let config_path = get_config_path();
    if !file_exists(&config_path) {
        return Vec::new();
    }

    let file = fs::File::open(&config_path).expect("Failed to open config file");
    let reader = io::BufReader::new(file);
    reader
        .lines()
        .filter_map(|line| {
            if let Ok(line) = line {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 4 {
                    return Some(Account {
                        name: parts[0].to_string(),
                        username: parts[1].to_string(),
                        email: parts[2].to_string(),
                        ssh_key: parts[3].to_string(),
                    });
                }
            }
            None
        })
        .collect()
}

pub fn list_accounts() {
    let accounts = load_accounts();
    if accounts.is_empty() {
        println!("No saved accounts.");
        return;
    }

    println!("🔹 Saved Git Accounts:");
    println!("----------------------------------------");
    println!("Account Name | Git Username | Email");
    println!("----------------------------------------");
    for acc in &accounts {
        println!("{} | {} | {}", acc.name, acc.username, acc.email);
    }
    println!("----------------------------------------");
}
