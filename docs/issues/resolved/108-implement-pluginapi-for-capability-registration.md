# Issue 108: Implement PluginApi for Capability Registration

## Summary
Implement the `PluginApi` struct that plugins use during registration to declare their capabilities. The API allows plugins to register channels, tools, CLI commands, model providers, and lifecycle hooks with the host application.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/api.rs`, `crates/aisopod-plugin/src/command.rs`

## Current Behavior
The `Plugin` trait (Issue 107) references a `PluginApi` parameter in its `register()` method, but the `PluginApi` type does not yet exist.

## Expected Behavior
A `PluginApi` struct provides methods `register_channel()`, `register_tool()`, `register_command()`, `register_provider()`, and `register_hook()` that plugins call during registration. A `PluginCommand` type represents CLI commands that plugins can contribute. All registered capabilities are collected and later consumed by the plugin registry.

## Impact
This is the primary interface between plugins and the host application. Without it, plugins cannot contribute any functionality to the system.

## Suggested Implementation
1. **Define `PluginCommand` in `command.rs`:**
   ```rust
   /// A CLI subcommand contributed by a plugin.
   pub struct PluginCommand {
       pub name: String,
       pub description: String,
       pub usage: String,
       pub require_auth: bool,
       pub handler: Box<dyn Fn(&[String]) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>,
   }
   ```
2. **Define `PluginApi` in `api.rs`:**
   ```rust
   use crate::PluginCommand;
   use std::sync::Arc;

   pub struct PluginApi {
       pub(crate) channels: Vec<Arc<dyn ChannelPlugin>>,
       pub(crate) tools: Vec<Arc<dyn Tool>>,
       pub(crate) commands: Vec<PluginCommand>,
       pub(crate) providers: Vec<Arc<dyn ModelProvider>>,
       pub(crate) hooks: Vec<(Hook, Arc<dyn HookHandler>)>,
   }

   impl PluginApi {
       pub fn new() -> Self {
           Self {
               channels: Vec::new(),
               tools: Vec::new(),
               commands: Vec::new(),
               providers: Vec::new(),
               hooks: Vec::new(),
           }
       }

       /// Register a channel implementation.
       pub fn register_channel(&mut self, channel: Arc<dyn ChannelPlugin>) {
           self.channels.push(channel);
       }

       /// Register a tool implementation.
       pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
           self.tools.push(tool);
       }

       /// Register a CLI subcommand.
       pub fn register_command(&mut self, command: PluginCommand) {
           self.commands.push(command);
       }

       /// Register a model provider.
       pub fn register_provider(&mut self, provider: Arc<dyn ModelProvider>) {
           self.providers.push(provider);
       }

       /// Register a lifecycle hook handler.
       pub fn register_hook(&mut self, hook: Hook, handler: Arc<dyn HookHandler>) {
           self.hooks.push((hook, handler));
       }
   }
   ```
3. **Re-export from `lib.rs`:**
   ```rust
   mod api;
   mod command;

   pub use api::PluginApi;
   pub use command::PluginCommand;
   ```
4. **Use trait objects** from the `aisopod-tools`, `aisopod-channel`, and `aisopod-provider` crates for the registration parameters. Import `Tool` from Issue 049, `ChannelPlugin` from Issue 089, and `ModelProvider` from Issue 038.

## Dependencies
- Issue 107 (Plugin trait and PluginMeta types)
- Issue 049 (Tool trait)
- Issue 089 (ChannelPlugin trait)
- Issue 038 (ModelProvider trait)

## Acceptance Criteria
- [x] `PluginApi` struct is defined with `register_channel()`, `register_tool()`, `register_command()`, `register_provider()`, and `register_hook()` methods
- [x] `PluginCommand` type supports name, description, usage, auth flag, and handler
- [x] Plugins can register channels using the `ChannelPlugin` trait
- [x] Plugins can register tools using the `Tool` trait
- [x] Plugins can register model providers using the `ModelProvider` trait
- [x] Plugins can register lifecycle hooks through the API
- [x] `cargo build -p aisopod-plugin` compiles without errors

## Resolution

The `PluginApi` struct was fully implemented as part of Issue 107 completion. The implementation includes:

- **PluginApi struct** (`crates/aisopod-plugin/src/api.rs`): Implemented with methods `register_channel()`, `register_tool()`, `register_command()`, `register_provider()`, and `register_hook()` for plugins to register their capabilities with the host application.

- **PluginCommand struct** (`crates/aisopod-plugin/src/command.rs`): Defined to represent CLI commands contributed by plugins, with fields for name, description, usage, require_auth flag, and handler.

- **Hook enum and HookHandler trait**: Defined for lifecycle event handling, allowing plugins to register hooks for various system events.

- **Updated Plugin trait**: Modified to use `PluginApi` as a concrete struct in the `register()` method signature.

- **Dependencies added**: The plugin crate now depends on `aisopod-channel`, `aisopod-provider`, and `aisopod-tools` crates for the trait objects used in registration methods.

- **Exports from lib.rs**: All types (`PluginApi`, `PluginCommand`, `Hook`, `HookHandler`, etc.) are properly exported from the crate's `lib.rs`.

- **Documentation**: All types and methods are properly documented with Rustdoc comments.

---
*Created: 2026-02-15*
*Resolved: 2026-02-23*
