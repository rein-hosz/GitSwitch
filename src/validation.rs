use crate::error::{GitSwitchError, Result};
use std::path::Path;
use std::process::Command;

/// Validate email format
pub fn validate_email(email: &str) -> Result<()> {
    if email_address::EmailAddress::is_valid(email) {
        Ok(())
    } else {
        Err(GitSwitchError::InvalidEmail {
            email: email.to_string(),
        })
    }
}

/// Validate SSH key format and permissions
pub fn validate_ssh_key(key_path: &Path) -> Result<()> {
    if !key_path.exists() {
        return Err(GitSwitchError::InvalidSshKey {
            message: format!("SSH key file not found: {}", key_path.display()),
        });
    }

    // Check file permissions (should be readable only by owner)
    let metadata = std::fs::metadata(key_path)?;
    let permissions = metadata.permissions();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = permissions.mode();
        if mode & 0o077 != 0 {
            return Err(GitSwitchError::InvalidSshKey {
                message: format!(
                    "SSH key has overly permissive permissions: {:o}. Should be 600 or similar.",
                    mode & 0o777
                ),
            });
        }
    }

    // Try to parse the SSH key to validate format
    let key_content = std::fs::read_to_string(key_path)?;
    
    // Basic validation for SSH key format
    if key_content.trim().is_empty() {
        return Err(GitSwitchError::InvalidSshKey {
            message: "SSH key file is empty".to_string(),
        });
    }

    // Check for common SSH key headers
    let valid_headers = [
        "-----BEGIN OPENSSH PRIVATE KEY-----",
        "-----BEGIN RSA PRIVATE KEY-----",
        "-----BEGIN DSA PRIVATE KEY-----",
        "-----BEGIN EC PRIVATE KEY-----",
        "-----BEGIN SSH2 ENCRYPTED PRIVATE KEY-----",
    ];

    if !valid_headers.iter().any(|header| key_content.contains(header)) {
        return Err(GitSwitchError::InvalidSshKey {
            message: "File does not appear to contain a valid SSH private key".to_string(),
        });
    }

    Ok(())
}

/// Comprehensive SSH key validation with enhanced security checks
// Comprehensive SSH key validation (currently unused but available for future use)
#[allow(dead_code)]
pub fn validate_ssh_key_comprehensive(key_path: &Path) -> Result<()> {
    // First run basic validation
    validate_ssh_key(key_path)?;
    
    // Enhanced validation
    let key_content = std::fs::read_to_string(key_path)
        .map_err(|e| GitSwitchError::Io(e))?;

    // Validate SSH private key content format
    validate_ssh_private_key_content(&key_content)?;

    // Check if corresponding public key exists and validate it
    let pub_key_path = format!("{}.pub", key_path.display());
    let pub_key_path = Path::new(&pub_key_path);
    
    if pub_key_path.exists() {
        validate_ssh_public_key_file(&pub_key_path)?;
        
        // Verify key pair matches
        verify_ssh_key_pair(key_path, &pub_key_path)?;
    } else {
        tracing::warn!("Public key file not found: {}", pub_key_path.display());
    }

    Ok(())
}

/// Validate SSH private key content format
#[allow(dead_code)]
fn validate_ssh_private_key_content(content: &str) -> Result<()> {
    let content = content.trim();
    
    // Check for OpenSSH format (preferred)
    if content.contains("-----BEGIN OPENSSH PRIVATE KEY-----") && 
       content.contains("-----END OPENSSH PRIVATE KEY-----") {
        return validate_openssh_private_key(content);
    }
    
    // Check for traditional formats
    let traditional_formats = [
        ("-----BEGIN RSA PRIVATE KEY-----", "-----END RSA PRIVATE KEY-----"),
        ("-----BEGIN DSA PRIVATE KEY-----", "-----END DSA PRIVATE KEY-----"),
        ("-----BEGIN EC PRIVATE KEY-----", "-----END EC PRIVATE KEY-----"),
        ("-----BEGIN SSH2 ENCRYPTED PRIVATE KEY-----", "-----END SSH2 ENCRYPTED PRIVATE KEY-----"),
    ];
    
    for (begin, end) in traditional_formats.iter() {
        if content.contains(begin) && content.contains(end) {
            return validate_traditional_private_key(content, begin, end);
        }
    }
    
    Err(GitSwitchError::InvalidSshKey {
        message: "File does not contain a recognized SSH private key format".to_string(),
    })
}

/// Validate OpenSSH format private key
#[allow(dead_code)]
fn validate_openssh_private_key(content: &str) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.len() < 3 {
        return Err(GitSwitchError::InvalidSshKey {
            message: "OpenSSH private key is too short".to_string(),
        });
    }
    
    // Check header and footer
    if lines[0] != "-----BEGIN OPENSSH PRIVATE KEY-----" {
        return Err(GitSwitchError::InvalidSshKey {
            message: "Invalid OpenSSH private key header".to_string(),
        });
    }
    
    if lines[lines.len() - 1] != "-----END OPENSSH PRIVATE KEY-----" {
        return Err(GitSwitchError::InvalidSshKey {
            message: "Invalid OpenSSH private key footer".to_string(),
        });
    }
    
    // Validate base64 content
    for (i, line) in lines.iter().enumerate().skip(1).take(lines.len() - 2) {
        if !is_valid_base64(line) {
            return Err(GitSwitchError::InvalidSshKey {
                message: format!("Invalid base64 content at line {}", i + 1),
            });
        }
    }
    
    Ok(())
}

/// Validate traditional format private key
#[allow(dead_code)]
fn validate_traditional_private_key(content: &str, begin: &str, end: &str) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.len() < 3 {
        return Err(GitSwitchError::InvalidSshKey {
            message: "Private key is too short".to_string(),
        });
    }
    
    // Check header and footer
    if lines[0] != begin {
        return Err(GitSwitchError::InvalidSshKey {
            message: "Invalid private key header".to_string(),
        });
    }
    
    if lines[lines.len() - 1] != end {
        return Err(GitSwitchError::InvalidSshKey {
            message: "Invalid private key footer".to_string(),
        });
    }
    
    // Validate base64 content
    for (i, line) in lines.iter().enumerate().skip(1).take(lines.len() - 2) {
        if !line.starts_with("Proc-Type:") && !line.starts_with("DEK-Info:") && !is_valid_base64(line) {
            return Err(GitSwitchError::InvalidSshKey {
                message: format!("Invalid content at line {}", i + 1),
            });
        }
    }
    
    Ok(())
}

/// Validate SSH public key file
#[allow(dead_code)]
fn validate_ssh_public_key_file(pub_key_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(pub_key_path)
        .map_err(|e| GitSwitchError::Io(e))?;
    
    validate_ssh_public_key_content(&content)
}

/// Validate SSH public key content
#[allow(dead_code)]
fn validate_ssh_public_key_content(content: &str) -> Result<()> {
    let content = content.trim();
    let parts: Vec<&str> = content.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err(GitSwitchError::InvalidSshKey {
            message: "Public key format is invalid (too few parts)".to_string(),
        });
    }
    
    // Check key type
    let key_type = parts[0];
    let valid_types = [
        "ssh-rsa", "ssh-dss", "ssh-ed25519", 
        "ecdsa-sha2-nistp256", "ecdsa-sha2-nistp384", "ecdsa-sha2-nistp521",
        "sk-ssh-ed25519@openssh.com", "sk-ecdsa-sha2-nistp256@openssh.com"
    ];
    
    if !valid_types.contains(&key_type) {
        return Err(GitSwitchError::InvalidSshKey {
            message: format!("Unsupported key type: {}", key_type),
        });
    }
    
    // Validate base64 key data
    let key_data = parts[1];
    if !is_valid_base64(key_data) {
        return Err(GitSwitchError::InvalidSshKey {
            message: "Invalid base64 encoding in public key".to_string(),
        });
    }
    
    // Optional: validate key strength
    validate_key_strength(key_type, key_data)?;
    
    Ok(())
}

/// Verify that private and public keys are a matching pair
#[allow(dead_code)]
fn verify_ssh_key_pair(private_key_path: &Path, public_key_path: &Path) -> Result<()> {
    // Use ssh-keygen to generate public key from private key and compare
    let output = std::process::Command::new("ssh-keygen")
        .arg("-y")
        .arg("-f")
        .arg(private_key_path)
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            let generated_public = String::from_utf8_lossy(&result.stdout);
            let stored_public = std::fs::read_to_string(public_key_path)
                .map_err(|e| GitSwitchError::Io(e))?;
            
            // Compare the key parts (ignore comments)
            let gen_parts: Vec<&str> = generated_public.trim().split_whitespace().take(2).collect();
            let stored_parts: Vec<&str> = stored_public.trim().split_whitespace().take(2).collect();
            
            if gen_parts.len() >= 2 && stored_parts.len() >= 2 {
                if gen_parts[0] != stored_parts[0] || gen_parts[1] != stored_parts[1] {
                    return Err(GitSwitchError::InvalidSshKey {
                        message: "Private and public keys do not match".to_string(),
                    });
                }
            }
        }
        Ok(result) => {
            let error = String::from_utf8_lossy(&result.stderr);
            return Err(GitSwitchError::InvalidSshKey {
                message: format!("Failed to verify key pair: {}", error),
            });
        }
        Err(_) => {
            tracing::warn!("ssh-keygen not available, skipping key pair verification");
        }
    }
    
    Ok(())
}

/// Validate key strength based on type and size
#[allow(dead_code)]
fn validate_key_strength(key_type: &str, key_data: &str) -> Result<()> {
    // Use base64 crate for decoding
    use base64::{Engine as _, engine::general_purpose};
    
    if let Ok(decoded) = general_purpose::STANDARD.decode(key_data) {
        match key_type {
            "ssh-rsa" => {
                // RSA keys should be at least 2048 bits
                if decoded.len() < 256 { // Rough estimate
                    tracing::warn!("RSA key appears to be less than 2048 bits, consider upgrading");
                }
            }
            "ssh-dss" => {
                tracing::warn!("DSA keys are deprecated and should be replaced with RSA or Ed25519");
            }
            "ssh-ed25519" => {
                // Ed25519 keys are always 256 bits and considered secure
            }
            _ if key_type.starts_with("ecdsa-") => {
                // ECDSA keys are generally secure with standard curves
            }
            _ => {
                // Other key types, no specific validation
            }
        }
    }
    
    Ok(())
}

/// Check if a string is valid base64
#[allow(dead_code)]
fn is_valid_base64(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    use base64::{Engine as _, engine::general_purpose};
    
    // Check for valid base64 characters
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        && general_purpose::STANDARD.decode(s).is_ok()
}

/// Check if Git is installed and accessible
pub fn validate_git_installation() -> Result<()> {
    match Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            tracing::info!("Git version: {}", version.trim());
            Ok(())
        }
        Ok(_) => Err(GitSwitchError::GitNotInstalled),
        Err(_) => Err(GitSwitchError::GitNotInstalled),
    }
}

/// Check if SSH agent is running
pub fn validate_ssh_agent() -> Result<()> {
    // Check if SSH_AUTH_SOCK environment variable is set
    if std::env::var("SSH_AUTH_SOCK").is_err() {
        return Err(GitSwitchError::SshAgentNotRunning);
    }

    // Try to list keys in the agent
    match Command::new("ssh-add").arg("-l").output() {
        Ok(output) if output.status.success() => {
            tracing::debug!("SSH agent is running");
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Could not open a connection to your authentication agent") {
                Err(GitSwitchError::SshAgentNotRunning)
            } else {
                // Agent is running but no keys loaded - that's okay
                Ok(())
            }
        }
        Err(_) => Err(GitSwitchError::SshAgentNotRunning),
    }
}

/// Validate account name (no special characters, reasonable length)
pub fn validate_account_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(GitSwitchError::Other(
            "Account name cannot be empty".to_string(),
        ));
    }

    if name.len() > 50 {
        return Err(GitSwitchError::Other(
            "Account name cannot be longer than 50 characters".to_string(),
        ));
    }

    // Check for invalid characters - allow spaces, alphanumeric, hyphens, and underscores
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ') {
        return Err(GitSwitchError::Other(
            "Account name can only contain alphanumeric characters, spaces, hyphens, and underscores".to_string(),
        ));
    }

    Ok(())
}

/// Validate username (basic checks)
pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        return Err(GitSwitchError::Other(
            "Username cannot be empty".to_string(),
        ));
    }

    if username.len() > 100 {
        return Err(GitSwitchError::Other(
            "Username cannot be longer than 100 characters".to_string(),
        ));
    }

    Ok(())
}

/// Comprehensive startup validation
pub fn validate_startup() -> Result<()> {
    tracing::info!("Performing startup validation...");
    
    validate_git_installation()?;
    
    // SSH agent validation is optional - warn but don't fail
    if let Err(e) = validate_ssh_agent() {
        tracing::warn!("SSH agent validation failed: {}", e);
        eprintln!("Warning: SSH agent is not running. Some features may not work properly.");
    }

    tracing::info!("Startup validation completed successfully");
    Ok(())
}
