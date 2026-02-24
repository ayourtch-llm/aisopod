//! # aisopod
//!
//! Main entry point for the aisopod application.

mod cli;
mod commands;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run_cli();
    Ok(())
}
