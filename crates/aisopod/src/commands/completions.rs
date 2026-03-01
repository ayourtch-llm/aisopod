//! Shell completion generation for the aisopod application.
//!
//! This module provides commands for generating shell completions for bash, zsh, fish, and PowerShell.

use crate::cli::Cli;
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};

/// Completions command arguments
#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    pub shell: Shell,
}

/// Generate shell completions
pub fn run(args: CompletionsArgs) {
    let mut cmd = Cli::command();
    generate(args.shell, &mut cmd, "aisopod", &mut std::io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completions_args() {
        use clap::ValueEnum;
        let shells = Shell::value_variants();
        assert_eq!(shells.len(), 5);
        assert!(shells.contains(&Shell::Bash));
        assert!(shells.contains(&Shell::Elvish));
        assert!(shells.contains(&Shell::Fish));
        assert!(shells.contains(&Shell::PowerShell));
        assert!(shells.contains(&Shell::Zsh));
    }
}
