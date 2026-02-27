# Issue 191: Write CLI Command Reference

## Summary
Create a complete CLI command reference documenting every command, subcommand, flag, and option available in the `aisopod` binary, with usage examples and environment variable interactions.

## Location
- Crate: N/A (documentation)
- File: `docs/book/src/cli-reference.md`

## Current Behavior
CLI commands are defined in code via `clap` or similar, but there is no user-facing reference document. Users must rely on `--help` output, which lacks examples and cross-references.

## Expected Behavior
A complete CLI reference at `docs/book/src/cli-reference.md` that documents every command with description, usage syntax, all flags/options, practical examples, and notes on environment variable overrides — serving as the authoritative CLI manual.

## Impact
The CLI is the primary interface for operators and developers. A complete reference reduces trial-and-error, improves discoverability of features, and complements `--help` with richer examples and context.

## Suggested Implementation

1. **Create** `docs/book/src/cli-reference.md` with the following structure:

2. **Top-level command overview:**
   ```markdown
   ## CLI Overview

   \```
   aisopod [OPTIONS] <COMMAND>

   Options:
     -c, --config <PATH>    Path to config file
     -v, --verbose           Increase log verbosity (-v, -vv, -vvv)
     -q, --quiet             Suppress non-error output
         --version            Print version
     -h, --help              Print help
   \```
   ```

3. **Each command documented with this template:**
   ```markdown
   ### `aisopod gateway`

   Manage the Aisopod gateway server.

   #### `aisopod gateway start`

   Start the gateway server.

   \```
   aisopod gateway start [OPTIONS]

   Options:
     --host <ADDR>       Listen address [default: 127.0.0.1]
     --port <PORT>       Listen port [default: 3080]
     --daemon            Run in background
     --pid-file <PATH>   Write PID file (implies --daemon)
   \```

   **Examples:**
   \```bash
   # Start with defaults
   aisopod gateway start

   # Start on all interfaces, custom port
   aisopod gateway start --host 0.0.0.0 --port 8080

   # Start as daemon
   aisopod gateway start --daemon --pid-file /var/run/aisopod.pid
   \```

   **Environment variables:**
   - `AISOPOD_HOST` overrides `--host`
   - `AISOPOD_PORT` overrides `--port`

   ---

   #### `aisopod gateway stop`

   Stop a running gateway.

   \```
   aisopod gateway stop [OPTIONS]

   Options:
     --pid-file <PATH>   PID file to read [default: /var/run/aisopod.pid]
     --force             Send SIGKILL instead of SIGTERM
   \```
   ```

4. **Document all command groups** following the pattern above:
   ```markdown
   ## Commands

   - [`aisopod init`](#aisopod-init) — Run onboarding wizard
   - [`aisopod gateway`](#aisopod-gateway) — Manage gateway server
     - `gateway start` / `gateway stop` / `gateway status`
   - [`aisopod chat`](#aisopod-chat) — Send a one-shot message
   - [`aisopod agent`](#aisopod-agent) — Manage agents
     - `agent list` / `agent create` / `agent delete` / `agent show`
   - [`aisopod channel`](#aisopod-channel) — Manage channels
     - `channel list` / `channel add` / `channel remove` / `channel test`
   - [`aisopod config`](#aisopod-config) — View/edit configuration
     - `config show` / `config edit` / `config validate`
   - [`aisopod doctor`](#aisopod-doctor) — Diagnose issues
   - [`aisopod migrate`](#aisopod-migrate) — Migrate from OpenClaw
   - [`aisopod completion`](#aisopod-completion) — Generate shell completions
   ```

5. **Shell Completion Setup section:**
   ```markdown
   ## Shell Completions

   Generate completion scripts for your shell:

   ### Bash
   \```bash
   aisopod completion bash > ~/.local/share/bash-completion/completions/aisopod
   \```

   ### Zsh
   \```bash
   aisopod completion zsh > ~/.zfunc/_aisopod
   # Add to .zshrc: fpath=(~/.zfunc $fpath); autoload -Uz compinit; compinit
   \```

   ### Fish
   \```bash
   aisopod completion fish > ~/.config/fish/completions/aisopod.fish
   \```

   ### PowerShell
   \```powershell
   aisopod completion powershell >> $PROFILE
   \```
   ```

6. **Update `SUMMARY.md`** to link to this page.

## Dependencies
- Issue 187 (documentation infrastructure)
- Issue 136 (CLI implementation complete — all commands available to document)

## Acceptance Criteria
- [ ] `docs/book/src/cli-reference.md` exists and is linked from `SUMMARY.md`
- [ ] Every CLI command and subcommand is documented
- [ ] Each command includes: description, usage syntax, all flags/options with defaults
- [ ] Each command has at least one practical example
- [ ] Environment variable overrides are noted where applicable
- [ ] Shell completion setup documented for Bash, Zsh, Fish, and PowerShell
- [ ] Reference matches actual `--help` output for all commands
- [ ] `mdbook build` succeeds with this page included

## Resolution

Implementation completed on 2026-02-27:

### What Was Implemented
Created complete CLI command reference documentation:

**1. Top-level Command Overview:**
- Documented main command structure with options
- Listed all global options (config path, verbosity, quiet mode, version, help)

**2. Command Documentation Template:**
- Used consistent template for all commands
- Included description, usage syntax, all flags/options with defaults
- Provided practical examples for each command

**3. Commands Documented:**
- `aisopod init` — Onboarding wizard
- `aisopod gateway` — Gateway server management (start, stop, status)
- `aisopod chat` — One-shot messaging
- `aisopod agent` — Agent management (list, create, delete, show)
- `aisopod channel` — Channel management (list, add, remove, test)
- `aisopod config` — Configuration viewing/editing (show, edit, validate)
- `aisopod doctor` — Diagnostic tool
- `aisopod migrate` — OpenClaw migration utility
- `aisopod completion` — Shell completion generation

**4. Environment Variable Integration:**
- Noted where environment variables override CLI flags
- Documented the precedence order (CLI > env var > config file)

**5. Shell Completion Setup:**
- Provided setup instructions for Bash, Zsh, Fish, and PowerShell
- Included specific commands and file paths for each shell

**6. Documentation Linking:**
- Updated SUMMARY.md with cli-reference.md link

### Files Created/Modified
- docs/book/src/cli-reference.md

### Test Results
- mdbook build docs/book: PASSED
- cargo build: PASSED

### Resolution Date
2026-02-27
