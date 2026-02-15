# Issue 131: Implement Channel Management Commands

## Summary
Implement the `aisopod channels` subcommands for listing configured channels and their status, and for running interactive setup wizards for supported channel types (Telegram, Discord, WhatsApp, Slack).

## Location
- Crate: `aisopod` (main binary crate)
- File: `src/commands/channels.rs`

## Current Behavior
The channels subcommand is a stub that panics with `todo!`. There is no CLI interface for managing messaging channels.

## Expected Behavior
Users can list all configured channels with their connection status and run interactive setup wizards that guide them through configuring each supported channel type with the required credentials and settings.

## Impact
Channel management is how users connect aisopod to external messaging platforms. Interactive setup wizards significantly reduce configuration errors and improve the onboarding experience.

## Suggested Implementation

1. Define the channels subcommand and its nested subcommands:

```rust
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ChannelsArgs {
    #[command(subcommand)]
    pub command: ChannelsCommands,
}

#[derive(Subcommand)]
pub enum ChannelsCommands {
    /// List configured channels and their status
    List,
    /// Interactive channel setup wizard
    Setup {
        /// Channel type to configure
        channel: String,
    },
}
```

2. Implement the list handler:

```rust
pub async fn list_channels(config_path: Option<String>) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    let channels = config.channels();

    if channels.is_empty() {
        println!("No channels configured. Run 'aisopod channels setup <channel>' to add one.");
        return Ok(());
    }

    println!("{:<15} {:<12} {:<30}", "Channel", "Status", "Details");
    println!("{}", "-".repeat(57));

    for channel in channels {
        let status = check_channel_status(&channel).await;
        println!("{:<15} {:<12} {:<30}", channel.channel_type, status, channel.description);
    }

    Ok(())
}
```

3. Implement the setup wizard with channel-specific prompts:

```rust
pub fn setup_channel(channel_type: &str, config_path: Option<String>) -> anyhow::Result<()> {
    let mut config = load_config(config_path)?;

    match channel_type {
        "telegram" => {
            println!("=== Telegram Bot Setup ===\n");
            println!("1. Open @BotFather on Telegram");
            println!("2. Send /newbot and follow the prompts");
            println!("3. Copy the bot token\n");

            let token = prompt_password("Bot token: ")?;
            let webhook = prompt_with_default("Webhook URL (leave blank for polling)", "")?;

            config.set_value("channels.telegram.token", &token)?;
            if !webhook.is_empty() {
                config.set_value("channels.telegram.webhook_url", &webhook)?;
            }
        }
        "discord" => {
            println!("=== Discord Bot Setup ===\n");
            println!("1. Go to https://discord.com/developers/applications");
            println!("2. Create a new application and add a bot");
            println!("3. Copy the bot token\n");

            let token = prompt_password("Bot token: ")?;
            let guild_id = prompt("Guild (server) ID: ")?;

            config.set_value("channels.discord.token", &token)?;
            config.set_value("channels.discord.guild_id", &guild_id)?;
        }
        "whatsapp" => {
            println!("=== WhatsApp Business Setup ===\n");
            let phone_id = prompt("Phone number ID: ")?;
            let token = prompt_password("Access token: ")?;
            let verify_token = prompt("Webhook verify token: ")?;

            config.set_value("channels.whatsapp.phone_number_id", &phone_id)?;
            config.set_value("channels.whatsapp.access_token", &token)?;
            config.set_value("channels.whatsapp.verify_token", &verify_token)?;
        }
        "slack" => {
            println!("=== Slack App Setup ===\n");
            let bot_token = prompt_password("Bot token (xoxb-...): ")?;
            let signing_secret = prompt_password("Signing secret: ")?;

            config.set_value("channels.slack.bot_token", &bot_token)?;
            config.set_value("channels.slack.signing_secret", &signing_secret)?;
        }
        _ => {
            anyhow::bail!("Unknown channel type: {}. Supported: telegram, discord, whatsapp, slack", channel_type);
        }
    }

    config.save()?;
    println!("\nChannel '{}' configured successfully!", channel_type);
    Ok(())
}
```

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 092 (channel registry)
- Issue 016 (configuration types)

## Acceptance Criteria
- [ ] `aisopod channels list` displays all configured channels with their status
- [ ] `aisopod channels setup telegram` runs the Telegram setup wizard
- [ ] `aisopod channels setup discord` runs the Discord setup wizard
- [ ] `aisopod channels setup whatsapp` runs the WhatsApp setup wizard
- [ ] `aisopod channels setup slack` runs the Slack setup wizard
- [ ] Unknown channel types produce a helpful error with supported options
- [ ] Credentials are stored securely and redacted in display

---
*Created: 2026-02-15*
