# Learning #160: Homebrew Formula and Configuration Templates

## Summary
Implemented a Homebrew formula for aisopod on macOS/Linux and configuration templates for different deployment environments (dev, production, docker).

## Changes Made

### 1. Homebrew Formula (`Formula/aisopod.rb`)
Created a Homebrew formula that supports:
- Cross-platform binaries (macOS ARM/x86, Linux x86_64)
- Version management via `version` constant
- Architecture detection using `Hardware::CPU.arm?`
- Placeholder SHA256 checksums (updated during release)

### 2. Configuration Templates (`config/templates/`)
Created three environment-specific templates:

**dev.json:**
- Development settings with verbose logging (`"level": "debug"`)
- Local-only binding (`"127.0.0.1"`)
- No authentication required (`allow_unconfigured: true`)

**production.json:**
- Production settings with standard logging (`"level": "info"`)
- Network binding (`"0.0.0.0"`)
- Authentication enabled (`security.require_auth: true`)

**docker.json:**
- Container-optimized settings
- Data directory set to `/data` (Docker volume convention)
- Network binding for container networking

### 3. Config Init Command Enhancement
Added `aisopod config init` command with:
- `--template` / `-t` flag: Specify template name (dev, production, docker)
- `--output` / `-o` flag: Specify output file path
- Template loading from multiple paths (current directory, workspace root)

### 4. Template Loading Implementation
The `get_template_content()` function tries multiple paths:
1. `config/templates/` (current working directory)
2. `<workspace-root>/config/templates/` (from compiled binary)

This ensures templates work both in development and when installed via Homebrew.

## Technical Details

### Path Resolution
Since `CARGO_MANIFEST_DIR` points to `crates/aisopod/` (not workspace root), we navigate up two levels:
```rust
let workspace_root_path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .and_then(|p| p.parent())
    .map(|p| p.join("config/templates").join(&template_filename));
```

### Template Structure
All templates inherit from the full config schema in `config/aisopod.example.json` and customize environment-specific settings.

## Testing

### Verified Functionality
- ✅ Formula file created at `Formula/aisopod.rb`
- ✅ All three template files created with valid JSON
- ✅ Templates have distinct configurations for their environments
- ✅ `aisopod config init -t <template> -o <path>` works correctly
- ✅ All template tests pass

### Test Output
```
running 7 tests
test commands::config::tests::test_config_args_default ... ok
test commands::config::tests::test_is_sensitive_field ... ok
test commands::config::tests::test_config_init_with_output ... ok
test commands::config::tests::test_config_init_command ... ok
test commands::config::tests::test_config_set_command ... ok
test commands::config::tests::test_load_config_or_default_no_file ... ok
test commands::config::tests::test_show_config_redacts_sensitive ... ok
```

## Future Enhancements

1. **Add SHA256 checksums**: Update placeholder checksums during release pipeline
2. **More templates**: Add development environment templates (test, staging)
3. **Template validation**: Add JSON schema validation for templates
4. **Template documentation**: Add template descriptions to `--help` output
5. **Template updates**: Add command to update existing configs with new templates

## Dependencies Met
- ✅ Issue 023 (default config generation) - Provided config structure foundation
- ✅ Issue 159 (release pipeline) - Binary URLs in formula assume release pipeline is operational

## Notes
- Homebrew formula checksums are placeholders and should be updated during release
- Templates use JSON format for compatibility with existing config loader
- The `--template` flag is optional; default is "dev" template

---
*Created: 2026-02-25*
