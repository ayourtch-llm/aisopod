# Issue 160: Create Homebrew Formula and Configuration Templates

## Summary
Create a Homebrew formula for installing aisopod on macOS and provide default configuration file templates for development, production, and Docker environments.

## Location
- Crate: `aisopod` (workspace root)
- Files:
  - `Formula/aisopod.rb` (or in a separate tap repository)
  - `config/templates/dev.json`
  - `config/templates/production.json`
  - `config/templates/docker.json`

## Current Behavior
There is no Homebrew formula for aisopod. Users must build from source or download binaries manually. No standard configuration templates exist for different deployment environments.

## Expected Behavior
A Homebrew formula that allows `brew install aisopod` (via a tap) and a set of configuration templates that users can copy and customize for their environment.

## Impact
Simplifies installation on macOS via Homebrew and provides ready-to-use configuration starting points for common deployment scenarios.

## Suggested Implementation

1. Create the Homebrew formula at `Formula/aisopod.rb`:

```ruby
class Aisopod < Formula
  desc "AI gateway and agent orchestration platform"
  homepage "https://github.com/AIsopod/aisopod"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/AIsopod/aisopod/releases/download/v#{version}/aisopod-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    else
      url "https://github.com/AIsopod/aisopod/releases/download/v#{version}/aisopod-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86"
    end
  end

  on_linux do
    url "https://github.com/AIsopod/aisopod/releases/download/v#{version}/aisopod-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"
  end

  def install
    bin.install "aisopod"
  end

  test do
    assert_match "aisopod", shell_output("#{bin}/aisopod --version")
  end
end
```

2. Create configuration templates directory and files:

**`config/templates/dev.json`** — Development config with verbose logging:
```json
{
  "gateway": {
    "bind_address": "127.0.0.1",
    "port": 18789,
    "allow_unconfigured": true
  },
  "logging": {
    "level": "debug"
  }
}
```

**`config/templates/production.json`** — Production config with security defaults:
```json
{
  "gateway": {
    "bind_address": "0.0.0.0",
    "port": 18789,
    "allow_unconfigured": false
  },
  "logging": {
    "level": "info"
  },
  "security": {
    "require_auth": true
  }
}
```

**`config/templates/docker.json`** — Docker-specific config:
```json
{
  "gateway": {
    "bind_address": "0.0.0.0",
    "port": 18789,
    "allow_unconfigured": true
  },
  "data_dir": "/data",
  "logging": {
    "level": "info"
  }
}
```

3. Update the `aisopod config init` command to optionally accept a `--template` flag that copies one of these templates.

## Dependencies
- Issue 023 (default config generation)
- Issue 159 (release pipeline for binary URLs)

## Acceptance Criteria
- [ ] Homebrew formula installs aisopod binary successfully via `brew install`
- [ ] Formula downloads the correct binary for the platform architecture
- [ ] `brew test aisopod` passes
- [ ] Configuration templates exist for `dev`, `production`, and `docker` environments
- [ ] Each template is valid JSON and contains environment-appropriate defaults
- [ ] Templates are documented in the deployment guide

---
*Created: 2026-02-15*
