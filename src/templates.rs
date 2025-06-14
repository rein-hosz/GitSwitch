use crate::config::Account;
use crate::error::{GitSwitchError, Result};
use std::collections::HashMap;

/// Account template for easy setup
#[derive(Debug, Clone)]
pub struct AccountTemplate {
    pub provider: String,
    pub ssh_test_host: String,
    pub ssh_key_upload_url: String,
    pub default_ssh_key_name: String,
}

/// Get available account templates
pub fn get_templates() -> HashMap<String, AccountTemplate> {
    let mut templates = HashMap::new();

    templates.insert(
        "github".to_string(),
        AccountTemplate {
            provider: "github".to_string(),
            ssh_test_host: "git@github.com".to_string(),
            ssh_key_upload_url: "https://github.com/settings/keys".to_string(),
            default_ssh_key_name: "id_rsa_github".to_string(),
        },
    );

    templates.insert(
        "gitlab".to_string(),
        AccountTemplate {
            provider: "gitlab".to_string(),
            ssh_test_host: "git@gitlab.com".to_string(),
            ssh_key_upload_url: "https://gitlab.com/-/profile/keys".to_string(),
            default_ssh_key_name: "id_rsa_gitlab".to_string(),
        },
    );

    templates.insert(
        "bitbucket".to_string(),
        AccountTemplate {
            provider: "bitbucket".to_string(),
            ssh_test_host: "git@bitbucket.org".to_string(),
            ssh_key_upload_url: "https://bitbucket.org/account/settings/ssh-keys/".to_string(),
            default_ssh_key_name: "id_rsa_bitbucket".to_string(),
        },
    );

    templates.insert(
        "azure".to_string(),
        AccountTemplate {
            provider: "azure".to_string(),
            ssh_test_host: "git@ssh.dev.azure.com".to_string(),
            ssh_key_upload_url: "https://dev.azure.com/_usersSettings/keys".to_string(),
            default_ssh_key_name: "id_rsa_azure".to_string(),
        },
    );

    templates
}

/// Create account from template
pub fn create_account_from_template(
    name: &str,
    username: &str,
    email: &str,
    template: &AccountTemplate,
) -> Account {
    Account {
        name: name.to_string(),
        username: username.to_string(),
        email: email.to_string(),
        ssh_key_path: format!("~/.ssh/{}", template.default_ssh_key_name),
        additional_ssh_keys: Vec::new(),
        provider: Some(template.provider.clone()),
        groups: Vec::new(),
    }
}

/// Get template by name
pub fn get_template(name: &str) -> Result<AccountTemplate> {
    let templates = get_templates();
    templates
        .get(name)
        .cloned()
        .ok_or_else(|| GitSwitchError::Other(format!("Unknown template: {}", name)))
}

/// List available templates
pub fn list_templates() {
    let templates = get_templates();

    println!("Available account templates:");
    println!("{}", "─".repeat(30));

    for (name, template) in &templates {
        println!("  {} - {}", name, template.provider);
        println!("    SSH Host: {}", template.ssh_test_host);
        println!("    Key Upload: {}", template.ssh_key_upload_url);
        println!();
    }
}
