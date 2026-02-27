# Issue 183: Create Channel Extension Template for Community Implementations

## Summary
Create a channel extension template and CLI scaffolding command that allows community developers to quickly create new channel implementations. The template provides a complete, compilable channel scaffold with all required trait implementations stubbed out.

## Location
- Crate: `aisopod-cli` (for the `aisopod channel create` command)
- File: `crates/aisopod-cli/src/commands/channel_create.rs`
- Template: `templates/channel/`

## Current Behavior
No channel template or scaffolding tool exists. Developers creating new channels must manually set up the crate structure, implement traits from scratch, and figure out the required module layout by reading existing channel implementations.

## Expected Behavior
After implementation:
- A template directory contains all files needed for a new channel crate.
- Running `aisopod channel create <name>` scaffolds a new channel project.
- The generated code compiles and includes stubbed ChannelPlugin trait implementation.
- README with documentation and getting-started instructions is included.

## Impact
Dramatically lowers the barrier for community channel contributions by providing a working starting point, consistent structure, and clear guidance on what needs to be implemented.

## Suggested Implementation

1. **Create template directory structure:**
   ```
   templates/channel/
   ├── Cargo.toml.tmpl
   ├── src/
   │   ├── lib.rs.tmpl
   │   ├── channel.rs.tmpl
   │   ├── config.rs.tmpl
   │   ├── outbound.rs.tmpl
   │   ├── gateway.rs.tmpl
   │   └── runtime.rs.tmpl
   └── README.md.tmpl
   ```

2. **Template: `Cargo.toml.tmpl`:**
   ```toml
   [package]
   name = "aisopod-channel-{{name}}"
   version = "0.1.0"
   edition = "2021"
   description = "{{display_name}} channel for aisopod"

   [dependencies]
   aisopod-channel-core = { path = "../aisopod-channel-core" }
   async-trait = "0.1"
   serde = { version = "1", features = ["derive"] }
   tokio = { version = "1", features = ["full"] }
   tracing = "0.1"
   ```

3. **Template: `src/lib.rs.tmpl`:**
   ```rust
   //! {{display_name}} channel implementation for aisopod.

   mod channel;
   mod config;
   mod outbound;
   mod gateway;
   mod runtime;

   pub use channel::{{pascal_name}}Channel;
   pub use config::{{pascal_name}}Config;

   use aisopod_channel_core::ChannelPlugin;

   /// Register this channel plugin with the aisopod runtime.
   pub fn register() -> Box<dyn ChannelPlugin> {
       Box::new(channel::{{pascal_name}}Channel::default())
   }
   ```

4. **Template: `src/channel.rs.tmpl`:**
   ```rust
   use async_trait::async_trait;
   use aisopod_channel_core::{ChannelPlugin, InboundMessage, OutboundMessage, ChannelError};
   use crate::config::{{pascal_name}}Config;

   #[derive(Default)]
   pub struct {{pascal_name}}Channel {
       config: Option<{{pascal_name}}Config>,
   }

   #[async_trait]
   impl ChannelPlugin for {{pascal_name}}Channel {
       fn name(&self) -> &str {
           "{{name}}"
       }

       async fn init(&mut self, config: serde_json::Value) -> Result<(), ChannelError> {
           let config: {{pascal_name}}Config = serde_json::from_value(config)
               .map_err(|e| ChannelError::Configuration(e.to_string()))?;
           self.config = Some(config);
           Ok(())
       }

       async fn connect(&mut self) -> Result<(), ChannelError> {
           // TODO: Implement connection logic
           tracing::info!("{{display_name}} channel connecting...");
           todo!("Implement connect for {{display_name}}")
       }

       async fn send(&self, msg: OutboundMessage) -> Result<(), ChannelError> {
           // TODO: Implement message sending
           todo!("Implement send for {{display_name}}")
       }

       async fn receive(&mut self) -> Result<InboundMessage, ChannelError> {
           // TODO: Implement message receiving
           todo!("Implement receive for {{display_name}}")
       }

       async fn disconnect(&mut self) -> Result<(), ChannelError> {
           // TODO: Implement disconnection logic
           tracing::info!("{{display_name}} channel disconnecting...");
           todo!("Implement disconnect for {{display_name}}")
       }
   }
   ```

5. **Template: `src/config.rs.tmpl`:**
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct {{pascal_name}}Config {
       // TODO: Add configuration fields specific to {{display_name}}
       // Example:
       // pub api_token: String,
       // pub server_url: String,
   }
   ```

6. **Template: `src/outbound.rs.tmpl`:**
   ```rust
   //! Outbound message handling for {{display_name}}.

   use aisopod_channel_core::OutboundMessage;

   /// Format an outbound message for the {{display_name}} platform.
   pub fn format_outbound(msg: &OutboundMessage) -> String {
       // TODO: Convert OutboundMessage to platform-specific format
       todo!()
   }
   ```

7. **Template: `src/gateway.rs.tmpl`** and **`src/runtime.rs.tmpl`** — similar stubs for gateway and runtime modules.

8. **Template: `README.md.tmpl`:**
   ```markdown
   # aisopod-channel-{{name}}

   {{display_name}} channel plugin for aisopod.

   ## Setup
   1. Add configuration to your aisopod config file
   2. Build: `cargo build -p aisopod-channel-{{name}}`

   ## Configuration
   ```toml
   [channels.{{name}}]
   # Add your configuration here
   ```

   ## Development
   See the [Channel Development Guide](../../docs/channel-development.md).
   ```

9. **CLI command** in `crates/aisopod-cli/src/commands/channel_create.rs`:
   ```rust
   use std::path::PathBuf;

   pub fn create_channel(name: &str) -> Result<(), Box<dyn std::error::Error>> {
       let pascal_name = to_pascal_case(name);
       let display_name = to_title_case(name);
       let target_dir = PathBuf::from(format!("crates/aisopod-channel-{}", name));

       if target_dir.exists() {
           return Err(format!("Channel crate already exists: {}", target_dir.display()).into());
       }

       std::fs::create_dir_all(target_dir.join("src"))?;

       // Read each template, substitute {{name}}, {{pascal_name}}, {{display_name}}
       // Write to target directory
       // Templates embedded via include_str! or read from templates/ dir

       println!("Created channel scaffold at {}", target_dir.display());
       println!("Next steps:");
       println!("  1. Edit src/config.rs to add your configuration fields");
       println!("  2. Implement connect/send/receive/disconnect in src/channel.rs");
       println!("  3. Run `cargo build -p aisopod-channel-{}` to verify", name);
       Ok(())
   }

   fn to_pascal_case(s: &str) -> String {
       s.split(&['-', '_'][..])
           .map(|word| {
               let mut chars = word.chars();
               match chars.next() {
                   None => String::new(),
                   Some(c) => c.to_uppercase().chain(chars).collect(),
               }
           })
           .collect()
   }
   ```

## Dependencies
- Issue 089: Channel trait definitions (ChannelPlugin trait)
- Issue 092: Channel lifecycle management
- Issue 122: Skill creator pattern (for template approach consistency)

## Acceptance Criteria
- [ ] Template directory contains all required files
- [ ] `aisopod channel create <name>` generates a new channel crate
- [ ] Generated code compiles without errors (with `todo!()` stubs)
- [ ] Generated code includes proper ChannelPlugin trait implementation
- [ ] Template variables (name, pascal_name, display_name) substitute correctly
- [ ] Generated README provides useful getting-started guidance
- [ ] CLI command validates input and prevents overwriting existing crates
- [ ] Unit tests for template substitution and CLI command

---
*Created: 2026-02-15*
