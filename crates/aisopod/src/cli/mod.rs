//! CLI commands for the aisopod application.
//!
//! This module provides the command-line interface for interacting with aisopod.
//! It uses clap for argument parsing and provides subcommands for various operations.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use aisopod_plugin::skills::{scaffold_skill, ScaffoldOptions, SkillCategory};

/// CLI arguments for the aisopod application.
#[derive(Parser)]
#[command(name = "aisopod")]
#[command(about = "Aisopod - An AI agent framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new skill
    ScaffoldSkill(ScaffoldSkillArgs),
}

/// Arguments for scaffolding a new skill.
#[derive(Parser)]
struct ScaffoldSkillArgs {
    /// The name of the skill (kebab-case)
    #[arg(short, long)]
    name: String,

    /// A brief description of the skill
    #[arg(short, long, default_value = "A new skill")]
    description: String,

    /// The category for this skill
    #[arg(short, long, default_value = "utility")]
    category: String,

    /// The output directory for the skill
    #[arg(short, long, default_value = "~/.aisopod/skills")]
    output_dir: String,
}

/// Main entry point for CLI processing.
pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ScaffoldSkill(args) => {
            run_scaffold_skill(args)?;
        }
    }

    Ok(())
}

/// Handle the scaffold-skill command.
fn run_scaffold_skill(args: ScaffoldSkillArgs) -> Result<()> {
    // Parse the category
    let category = match args.category.to_lowercase().as_str() {
        "messaging" => SkillCategory::Messaging,
        "productivity" => SkillCategory::Productivity,
        "aiml" | "ai-ml" | "ai_ml" => SkillCategory::AiMl,
        "integration" => SkillCategory::Integration,
        "utility" => SkillCategory::Utility,
        "system" => SkillCategory::System,
        _ => {
            eprintln!(
                "Unknown category '{}'. Valid categories: messaging, productivity, ai-ml, integration, utility, system",
                args.category
            );
            return Err(anyhow::anyhow!("Invalid category"));
        }
    };

    // Parse and expand the output directory
    let output_dir = expand_path(&args.output_dir);

    // Create scaffold options
    let opts = ScaffoldOptions {
        name: args.name,
        description: args.description,
        category,
        output_dir,
    };

    // Generate the skill
    match scaffold_skill(&opts) {
        Ok(skill_dir) => {
            println!("Created skill at: {:?}", skill_dir);
            println!("\nNext steps:");
            println!("1. Edit {:?}", skill_dir.join("src/lib.rs"));
            println!("   Implement your skill's tools and system prompt.");
            println!("2. Update {:?}", skill_dir.join("skill.toml"));
            println!("   Add any required environment variables or binaries.");
            println!("3. Place this directory in ~/.aisopod/skills/ to make it available.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to scaffold skill: {}", e);
            Err(e)
        }
    }
}

/// Expand a path, handling ~ and relative paths.
fn expand_path(path: &str) -> PathBuf {
    let expanded = if path.starts_with('~') {
        let home = std::env::var("HOME")
            .unwrap_or_else(|_| ".".to_string());
        path.replacen('~', &home, 1)
    } else {
        path.to_string()
    };
    PathBuf::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path_home() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let result = expand_path("~/test");
        assert_eq!(result, PathBuf::from(format!("{}/test", home)));
    }

    #[test]
    fn test_expand_path_absolute() {
        let result = expand_path("/tmp/test");
        assert_eq!(result, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_expand_path_relative() {
        let result = expand_path("relative/path");
        assert_eq!(result, PathBuf::from("relative/path"));
    }
}
