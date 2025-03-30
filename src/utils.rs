use std::process::Command;

pub fn run_command(command: &str, args: &[&str]) -> bool {
    println!("$ {} {}", command, args.join(" "));
    let status = Command::new(command)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("❌ Failed to execute command '{}': {}", command, e);
            std::process::exit(1);
        });

    if !status.success() {
        eprintln!("❌ Error running {} {:?}", command, args);
        return false;
    }
    true
}

pub fn file_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}
