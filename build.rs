use std::env;
use std::process::Command;

fn main() {
    // Ensure cargo reruns the build script if these change
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");

    let cargo_pkg_version =
        env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set in build script");

    let mut git_details_str = "".to_string();
    match Command::new("git")
        .args(["describe", "--tags", "--dirty", "--always"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let details = String::from_utf8(output.stdout)
                .unwrap_or_default()
                .trim()
                .to_string();
            // Ensure the output is not empty and not a git error message like "fatal: ..."
            if !details.is_empty() && !details.starts_with("fatal:") {
                git_details_str = details;
            }
        }
        _ => {
            // On any error or if git describe doesn't yield a useful string, git_details_str remains empty
        }
    }

    // This will be used for clap's `version` attribute (e.g., "0.1.0")
    println!("cargo:rustc-env=APP_VERSION={}", cargo_pkg_version);

    // This will be used for clap's `long_version` attribute
    let app_long_version_str = if git_details_str.is_empty() {
        cargo_pkg_version.clone() // Fallback to just package version if no git details
    } else {
        format!("{} (git: {})", cargo_pkg_version, git_details_str)
    };
    println!("cargo:rustc-env=APP_LONG_VERSION={}", app_long_version_str);
}
