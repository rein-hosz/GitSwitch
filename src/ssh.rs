use crate::utils::{file_exists, run_command};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn get_ssh_config_path() -> String {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".ssh")
        .join("config")
        .to_string_lossy()
        .into_owned()
}

pub fn generate_ssh_key(identity_file: &str) {
    let expanded_path = if identity_file.starts_with("~") {
        let home = dirs::home_dir().expect("Could not determine home directory");
        home.join(&identity_file[2..])
            .to_string_lossy()
            .into_owned()
    } else {
        identity_file.to_string()
    };

    if file_exists(&expanded_path) {
        println!("✅ SSH key already exists: {}", identity_file);
        return;
    }

    // Ensure the directory exists
    if let Some(parent) = Path::new(&expanded_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Failed to create SSH directory");
        }
    }

    println!("🔑 Generating SSH key: {}", identity_file);
    run_command(
        "ssh-keygen",
        &["-t", "rsa", "-b", "4096", "-f", &expanded_path, "-N", ""],
    );
}

pub fn display_public_key(identity_file: &str) {
    let public_key_path = format!("{}.pub", shellexpand::tilde(identity_file));

    match File::open(&public_key_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                println!("{}", contents.trim());
            } else {
                println!(
                    "❌ Failed to read public key file. Please check the file at: {}",
                    public_key_path
                );
            }
        }
        Err(_) => {
            println!("❌ Public key file not found at: {}", public_key_path);
        }
    }
}

pub fn update_ssh_config(name: &str, identity_file: &str) -> io::Result<()> {
    let config_entry = format!(
        "\n# {} GitHub Account\nHost github-{}\n    HostName github.com\n    User git\n    IdentityFile {}\n",
        name, name, identity_file
    );

    let expanded_path = get_ssh_config_path();
    let path = Path::new(&expanded_path);

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    file.write_all(config_entry.as_bytes())?;
    println!("✅ Updated SSH config for account: {}", name);
    Ok(())
}

pub fn remove_ssh_config_entry(name: &str) -> io::Result<()> {
    let config_path_str = get_ssh_config_path();
    let path = Path::new(&config_path_str);

    if !path.exists() {
        println!("ℹ️ SSH config file not found, nothing to remove for account '{}'.", name);
        return Ok(());
    }

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut new_content = String::new();
    let mut in_matching_block = false;
    let mut lines_to_skip = 0;

    let entry_header = format!("# {} GitHub Account", name);

    for line in reader.lines() {
        let line = line?;
        if line.trim() == entry_header {
            in_matching_block = true;
            lines_to_skip = 4; // Header + 3 lines of config
        }

        if in_matching_block {
            if lines_to_skip > 0 {
                lines_to_skip -= 1;
                // Skip this line
            } else {
                in_matching_block = false;
                // Add this line as it's after the block to remove
                new_content.push_str(&line);
                new_content.push('\n');
            }
        } else {
            new_content.push_str(&line);
            new_content.push('\n');
        }
    }

    // Remove trailing newline if it's the only thing left, or multiple newlines at the end
    while new_content.ends_with("\n\n") {
        new_content.pop();
    }
    if new_content == "\n" { // If only a single newline is left
        new_content.clear();
    }


    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(new_content.as_bytes())?;
    println!("🗑️ SSH config entry for '{}' removed.", name);
    Ok(())
}

pub fn delete_ssh_key_files(identity_file_base: &str) -> io::Result<()> {
    let base_path = shellexpand::tilde(identity_file_base).to_string();
    let private_key_path = Path::new(&base_path);
    let public_key_path = Path::new(&format!("{}.pub", base_path));

    if private_key_path.exists() {
        fs::remove_file(private_key_path)?;
        println!("🗑️ Deleted private SSH key: {}", private_key_path.display());
    }

    if public_key_path.exists() {
        fs::remove_file(public_key_path)?;
        println!("🗑️ Deleted public SSH key: {}", public_key_path.display());
    }

    Ok(())
}

pub fn add_ssh_key(key_path: &str) -> bool {
    let home = dirs::home_dir().expect("Could not determine home directory");
    let expanded_path = if key_path.starts_with("~") {
        home.join(&key_path[2..]).to_string_lossy().into_owned()
    } else {
        key_path.to_string()
    };

    if !file_exists(&expanded_path) {
        println!("❌ SSH key not found: {}", key_path);
        return false;
    }

    // Handle Windows differently if needed
    if cfg!(windows) {
        println!("🔑 Adding SSH key to agent (Windows): {}", key_path);
        // Use Windows-specific approach or skip if not needed
        true
    } else {
        println!("🔑 Adding SSH key to agent: {}", key_path);
        run_command("ssh-add", &[&expanded_path])
    }
}
