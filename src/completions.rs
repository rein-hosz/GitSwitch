use clap::Command;
use clap_complete::{Shell, generate};
use std::io;

/// Generate shell completion scripts
pub fn generate_completions(shell: Shell, cmd: &mut Command) {
    generate(shell, cmd, "git-switch", &mut io::stdout());
}

/// Print installation instructions for each shell
pub fn print_installation_instructions(shell: Shell) {
    match shell {
        Shell::Bash => {
            println!("# To install bash completions, add the following to your ~/.bashrc:");
            println!("# source <(git-switch completions bash)");
            println!("# Or save to a file and source it:");
            println!(
                "# git-switch completions bash > ~/.local/share/bash-completion/completions/git-switch"
            );
        }
        Shell::Zsh => {
            println!("# To install zsh completions, add the following to your ~/.zshrc:");
            println!("# autoload -U compinit");
            println!("# compinit");
            println!("# source <(git-switch completions zsh)");
            println!("# Or save to a file in your fpath:");
            println!(
                "# git-switch completions zsh > ~/.local/share/zsh/site-functions/_git-switch"
            );
        }
        Shell::Fish => {
            println!("# To install fish completions:");
            println!("# git-switch completions fish > ~/.config/fish/completions/git-switch.fish");
        }
        Shell::PowerShell => {
            println!("# To install PowerShell completions, add to your PowerShell profile:");
            println!("# git-switch completions powershell | Out-String | Invoke-Expression");
        }
        Shell::Elvish => {
            println!("# To install Elvish completions:");
            println!("# git-switch completions elvish > ~/.elvish/lib/git-switch.elv");
        }
        _ => {
            println!("# Installation instructions not available for this shell");
        }
    }
}
