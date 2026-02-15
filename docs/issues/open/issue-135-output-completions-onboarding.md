# Issue 135: Implement Output Formatting, Shell Completions, and Interactive Onboarding

## Summary
Implement colored terminal output, table formatting for list commands, JSON output mode, progress indicators, shell completion generation, and an interactive onboarding wizard for first-time users.

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/output.rs`, `src/commands/completions.rs`, `src/commands/onboarding.rs`

## Current Behavior
Command output is plain unformatted text. There are no shell completions, no JSON output mode, and no guided onboarding experience for new users.

## Expected Behavior
All list commands display results in formatted tables. Colored output highlights important information. The `--json` flag produces machine-readable JSON output. Long operations show progress indicators. Shell completions can be generated for bash, zsh, fish, and PowerShell. First-time users are guided through an onboarding wizard.

## Impact
Output formatting significantly improves the user experience and readability. Shell completions improve CLI discoverability. The onboarding wizard reduces time-to-first-message for new users.

## Suggested Implementation

1. Add output formatting dependencies to `Cargo.toml`:

```toml
[dependencies]
colored = "2"
comfy-table = "7"
indicatif = "0.17"
serde_json = "1"
```

2. Create `src/output.rs` with formatting utilities:

```rust
use colored::Colorize;
use comfy_table::{Table, presets::UTF8_FULL};

pub struct Output {
    json_mode: bool,
}

impl Output {
    pub fn new(json_mode: bool) -> Self {
        Self { json_mode }
    }

    pub fn print_table(&self, headers: &[&str], rows: Vec<Vec<String>>) {
        if self.json_mode {
            let json_rows: Vec<_> = rows.iter().map(|row| {
                headers.iter().zip(row.iter())
                    .map(|(h, v)| (h.to_string(), v.clone()))
                    .collect::<std::collections::HashMap<_, _>>()
            }).collect();
            println!("{}", serde_json::to_string_pretty(&json_rows).unwrap());
            return;
        }

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(headers);
        for row in rows {
            table.add_row(row);
        }
        println!("{table}");
    }

    pub fn success(&self, msg: &str) {
        if self.json_mode {
            println!(r#"{{"status":"success","message":"{}"}}"#, msg);
        } else {
            println!("{} {}", "✓".green(), msg);
        }
    }

    pub fn error(&self, msg: &str) {
        if self.json_mode {
            eprintln!(r#"{{"status":"error","message":"{}"}}"#, msg);
        } else {
            eprintln!("{} {}", "✗".red(), msg);
        }
    }

    pub fn info(&self, msg: &str) {
        if self.json_mode {
            println!(r#"{{"status":"info","message":"{}"}}"#, msg);
        } else {
            println!("{} {}", "ℹ".blue(), msg);
        }
    }

    pub fn warning(&self, msg: &str) {
        if self.json_mode {
            println!(r#"{{"status":"warning","message":"{}"}}"#, msg);
        } else {
            println!("{} {}", "⚠".yellow(), msg);
        }
    }
}
```

3. Add progress indicator helpers:

```rust
use indicatif::{ProgressBar, ProgressStyle};

pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ ")
    );
    pb.set_message(message.to_string());
    pb
}
```

4. Implement shell completions in `src/commands/completions.rs`:

```rust
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};
use crate::cli::Cli;

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    pub shell: Shell,
}

pub fn run(args: CompletionsArgs) {
    let mut cmd = Cli::command();
    generate(args.shell, &mut cmd, "aisopod", &mut std::io::stdout());
}
```

Add the dependency:

```toml
clap_complete = "4"
```

5. Implement the onboarding wizard in `src/commands/onboarding.rs`:

```rust
pub fn run_onboarding(config_path: Option<String>) -> anyhow::Result<()> {
    println!("{}", "Welcome to aisopod!".bold());
    println!("Let's get you set up.\n");

    // Step 1: Auth setup
    println!("Step 1: Configure your AI provider\n");
    crate::commands::auth::run_auth_setup(config_path.clone())?;

    // Step 2: Model selection
    println!("\nStep 2: Choose your default model\n");
    let model = prompt_with_default("Default model", "gpt-4")?;
    let mut config = load_config(config_path.clone())?;
    config.set_value("models.default", &model)?;

    // Step 3: Channel setup (optional)
    println!("\nStep 3: Set up a messaging channel (optional)\n");
    let setup_channel = prompt("Would you like to set up a channel? (yes/no): ")?;
    if setup_channel == "yes" {
        crate::commands::channels::setup_channel(
            &prompt_select("Channel", &["telegram", "discord", "whatsapp", "slack"])?,
            config_path.clone(),
        )?;
    }

    // Step 4: Send first message
    println!("\nStep 4: Send your first message!\n");
    let first_msg = prompt("Type a message (or press Enter to skip): ")?;
    if !first_msg.is_empty() {
        println!("\nTo send this message, start the gateway and run:");
        println!("  aisopod gateway &");
        println!("  aisopod message \"{}\"", first_msg);
    }

    println!("\n{}", "Setup complete! 🎉".green().bold());
    println!("\nNext steps:");
    println!("  aisopod gateway     - Start the gateway server");
    println!("  aisopod message     - Send a message");
    println!("  aisopod status      - Check system status");
    println!("  aisopod --help      - See all commands");

    Ok(())
}
```

## Dependencies
- Issue 124 (clap CLI framework)

## Acceptance Criteria
- [ ] List commands display results in formatted tables using comfy-table
- [ ] Output is colored with success (green), error (red), info (blue), warning (yellow)
- [ ] `--json` flag on any command produces valid JSON output
- [ ] Long operations show a spinner or progress bar
- [ ] `aisopod completions bash` generates valid bash completions
- [ ] `aisopod completions zsh` generates valid zsh completions
- [ ] `aisopod completions fish` generates valid fish completions
- [ ] `aisopod completions powershell` generates valid PowerShell completions
- [ ] First run triggers onboarding wizard that guides through auth, model, channel, first message
- [ ] Colors are automatically disabled when output is not a TTY

---
*Created: 2026-02-15*
