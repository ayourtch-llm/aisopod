# Issue 130: Implement Model Management Commands

## Summary
Implement the `aisopod models` subcommands for listing available models across all configured providers and switching the primary model for the default agent.

## Location
- Crate: `aisopod` (main binary crate)
- File: `crates/aisopod/src/commands/models.rs`

## Current Behavior
The models subcommand is a stub that panics with `todo!`. There is no CLI interface for discovering or switching models.

## Expected Behavior
Users can list all available models from all configured providers (e.g., OpenAI, Anthropic, local) and switch the primary model used by the default agent with a single command.

## Impact
Model management is essential for users who want to experiment with different models or switch providers. This avoids manual config file editing for a common operation.

## Dependencies
- Issue 124 (clap CLI framework)
- Issue 047 (model discovery)
- Issue 016 (configuration types)

## Acceptance Criteria
- [x] `aisopod models list` displays all available models grouped by provider
- [x] `aisopod models list --provider openai` filters to a specific provider
- [x] `aisopod models switch gpt-4` updates the default agent's model
- [x] Currently active model is indicated in the list output
- [x] Error message when switching to an unknown model
- [x] JSON output mode (`--json`) returns structured model data

## Resolution

The model management commands have been fully implemented in `crates/aisopod/src/commands/models.rs`:

### Commands Implemented

1. **`aisopod models list`** - Lists all available models across all configured providers
   - Supports `--provider <name>` to filter by specific provider
   - Supports `--json` for structured JSON output
   - Displays models grouped by provider with (default) marker for the active model

2. **`aisopod models switch <model>`** - Switches the primary model for the default agent
   - Validates the model exists in any configured provider
   - Updates the default agent's model in the configuration file
   - Supports `--json` for structured JSON output
   - Provides helpful error messages when model is not found

### Implementation Details

The implementation includes:
- `ModelsArgs` and `ModelsCommands` clap structs for CLI parsing
- `list_models()` async function that discovers models via `ModelCatalog`
- `switch_model()` async function that updates configuration
- Support for multiple providers: OpenAI, Anthropic, Gemini, Bedrock, Ollama
- Proper error handling with detailed error messages
- JSON output mode for programmatic consumption
- Unit tests for argument parsing

The commands integrate with the existing CLI framework in `crates/aisopod/src/cli.rs` and the main entry point in `crates/aisopod/src/main.rs`.

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
