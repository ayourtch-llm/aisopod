# Issue 196: Write Troubleshooting Guide and Generate API Docs from Code

## Summary
Create a troubleshooting guide covering common errors, diagnostic tools, and performance tuning, and set up automated `cargo doc` generation to publish Rust API documentation alongside the mdBook site.

## Location
- Crate: N/A (documentation and CI)
- File: `docs/book/src/troubleshooting.md`, `.github/workflows/ci.yml` (docs job)

## Current Behavior
No troubleshooting documentation exists. Users encountering errors have no self-service path to resolution. Rust API documentation is not generated or published automatically.

## Expected Behavior
A troubleshooting guide at `docs/book/src/troubleshooting.md` that covers common errors, the `aisopod doctor` diagnostic tool, log analysis, channel-specific issues, and performance tuning. Additionally, `cargo doc` output is automatically generated and published alongside the mdBook documentation site.

## Impact
Troubleshooting documentation reduces support burden and improves user self-sufficiency. Auto-generated API docs ensure that Rust-level documentation stays current with every release, serving developers building on or contributing to Aisopod.

## Suggested Implementation

1. **Create** `docs/book/src/troubleshooting.md` with the following sections:

2. **Common Errors and Solutions section:**
   ```markdown
   ## Common Errors

   ### `Error: Connection refused (port 3080)`
   **Cause:** Gateway is not running or listening on a different port.
   **Solution:**
   \```bash
   # Check if gateway is running
   aisopod gateway status

   # Start if not running
   aisopod gateway start

   # Verify port
   ss -tlnp | grep 3080
   \```

   ### `Error: 401 Unauthorized`
   **Cause:** Missing or invalid authentication token.
   **Solution:**
   - Verify token in request: `Authorization: Bearer <token>`
   - Check config: `aisopod config show | grep auth`
   - If using env var: `echo $AISOPOD_AUTH_TOKEN`

   ### `Error: 502 Upstream Error`
   **Cause:** LLM provider returned an error (rate limit, invalid key, outage).
   **Solution:**
   \```bash
   # Check provider status
   aisopod doctor --check-providers

   # Test API key directly
   curl https://api.openai.com/v1/models \
     -H "Authorization: Bearer $AISOPOD_OPENAI_API_KEY"
   \```

   ### `Error: Channel connection failed`
   **Cause:** Invalid channel token or network issue.
   **Solution:**
   \```bash
   aisopod channel test telegram
   \```

   ### `Error: Sandbox execution timeout`
   **Cause:** Code execution exceeded the configured timeout.
   **Solution:**
   - Increase timeout: `tools.sandbox.timeout = 60`
   - Check if Docker daemon is running: `docker info`

   ### `Error: Config parse error`
   **Cause:** Invalid TOML syntax in config file.
   **Solution:**
   \```bash
   aisopod config validate
   \```
   ```

3. **Diagnostic Commands section:**
   ```markdown
   ## Diagnostic Commands

   ### `aisopod doctor`
   Runs a comprehensive health check:
   \```bash
   aisopod doctor
   \```

   Output:
   \```
   ✅ Configuration: valid
   ✅ Authentication: token mode configured
   ✅ OpenAI API: connected (gpt-4o available)
   ✅ Anthropic API: connected (claude-3-opus available)
   ⚠️  Telegram channel: token valid, webhook not set
   ✅ Sandbox: Docker available, image pulled
   ✅ Storage: sessions.db accessible (142 sessions)
   ❌ Disk space: less than 100MB free — consider cleanup
   \```

   ### Targeted checks:
   \```bash
   aisopod doctor --check-config       # Config file only
   aisopod doctor --check-providers    # LLM provider connectivity
   aisopod doctor --check-channels     # Channel connectivity
   aisopod doctor --check-data         # Storage integrity
   aisopod doctor --check-sandbox      # Sandbox runtime
   \```
   ```

4. **Log Analysis section:**
   ```markdown
   ## Log Analysis

   ### Setting Log Level
   \```bash
   # Environment variable
   AISOPOD_LOG=debug aisopod gateway start

   # Granular control
   AISOPOD_LOG=aisopod_gateway=debug,aisopod_agent=info aisopod gateway start
   \```

   ### Log Levels
   | Level   | Use Case                                    |
   |---------|---------------------------------------------|
   | `error` | Production — only errors                    |
   | `warn`  | Production — errors + warnings              |
   | `info`  | Default — request/response summaries        |
   | `debug` | Development — detailed request flow         |
   | `trace` | Debugging — full message payloads           |

   ### Key Log Patterns
   \```bash
   # Find failed requests
   grep "ERROR" /var/log/aisopod.log

   # Trace a specific request
   grep "request_id=abc123" /var/log/aisopod.log

   # Monitor channel activity
   AISOPOD_LOG=aisopod_channel=debug aisopod gateway start 2>&1 | grep channel
   \```

   ### Structured Logging
   Aisopod outputs structured JSON logs when `AISOPOD_LOG_FORMAT=json`:
   \```json
   {"timestamp":"2026-01-15T10:30:00Z","level":"INFO","target":"aisopod_gateway","message":"Request processed","request_id":"abc123","duration_ms":250}
   \```
   ```

5. **Channel-Specific Troubleshooting section:**
   ```markdown
   ## Channel Troubleshooting

   ### Telegram
   - **Bot not responding:** Verify webhook is set — `aisopod channel test telegram`
   - **Duplicate messages:** Ensure only one instance is running
   - **Media not supported:** Check agent skills include `file_read`

   ### Discord
   - **Bot offline:** Check bot token and gateway intents
   - **Missing permissions:** Bot needs `Send Messages`, `Read Message History`
   - **Rate limited:** Discord has strict rate limits — reduce message frequency

   ### WhatsApp
   - **Webhook verification failed:** Ensure `verify_token` matches
   - **Messages not delivered:** Check WhatsApp Business API status
   - **Template messages required:** First outbound messages need approved templates

   ### Slack
   - **Socket Mode disconnecting:** Check `app_token` (starts with `xapp-`)
   - **Bot not responding to DMs:** Add `im:history` scope
   - **Event subscriptions:** Ensure request URL is reachable from Slack
   ```

6. **Performance Tuning section:**
   ```markdown
   ## Performance Tuning

   ### Gateway Tuning
   \```toml
   [server]
   workers = 8              # match CPU cores for I/O-bound workloads
   request_timeout = 60     # increase for slow models
   max_connections = 1024   # increase for high-traffic deployments
   \```

   ### Memory Usage
   - **Reduce session cache:** `storage.session_cache_size = 100`
   - **Limit concurrent agents:** Each agent holds model context in memory
   - **Monitor with:** `aisopod gateway status --verbose`

   ### Latency Optimization
   - Use streaming (`stream: true`) for faster time-to-first-token
   - Choose geographically close LLM providers
   - Enable connection pooling (default: on)
   - Use fallback models for resilience, not for every request

   ### Throughput Scaling
   - Run multiple Aisopod instances behind a load balancer
   - Use shared storage backend (PostgreSQL) for session persistence
   - Separate channel webhook receivers from chat API endpoints
   ```

7. **Cargo Doc Generation section:**
   ```markdown
   ## Rust API Documentation

   Auto-generated Rust API documentation is available at
   `https://docs.aisopod.dev/api/`.

   ### Generate Locally
   \```bash
   cargo doc --workspace --no-deps --document-private-items
   open target/doc/aisopod/index.html
   \```
   ```

8. **Add `cargo doc` to CI** (in `.github/workflows/ci.yml`):
   ```yaml
   docs:
     name: Build Documentation
     runs-on: ubuntu-latest
     steps:
       - uses: actions/checkout@v4
       - uses: dtolnay/rust-toolchain@stable

       - name: Build mdBook
         run: |
           cargo install mdbook --version 0.4.37 --locked
           mdbook build docs/book

       - name: Generate Rust API docs
         run: cargo doc --workspace --no-deps

       - name: Combine documentation
         run: |
           mkdir -p docs/book/build/api
           cp -r target/doc/* docs/book/build/api/

       - name: Upload documentation
         uses: actions/upload-artifact@v4
         with:
           name: documentation
           path: docs/book/build
   ```

9. **Update `SUMMARY.md`** to link to this page.

## Dependencies
- Issue 187 (documentation infrastructure — mdBook setup)
- Issue 188 (getting started guide)
- Issue 189 (configuration guide)
- Issue 190 (agents, channels, skills guide)
- Issue 191 (CLI reference)
- Issue 192 (API reference)
- Issue 193 (developer guide)
- Issue 194 (migration guide)
- Issue 195 (security and deployment guide)

## Acceptance Criteria
- [ ] `docs/book/src/troubleshooting.md` exists and is linked from `SUMMARY.md`
- [ ] At least 6 common errors documented with causes and solutions
- [ ] `aisopod doctor` command and all `--check-*` flags documented
- [ ] Log level configuration and structured logging documented
- [ ] Channel-specific troubleshooting for all Tier 1 channels (Telegram, Discord, WhatsApp, Slack)
- [ ] Performance tuning covers gateway, memory, latency, and throughput
- [ ] `cargo doc --workspace --no-deps` generates documentation without errors
- [ ] CI pipeline combines mdBook and cargo doc output into a single artifact
- [ ] `mdbook build` succeeds with this page included

## Resolution

Implementation completed on 2026-02-27:

### What Was Implemented
Created comprehensive troubleshooting guide and set up Rust API documentation:

**1. Common Errors Documentation:**
- Documented "Connection refused" errors with gateway status commands
- Documented "401 Unauthorized" with authentication troubleshooting
- Documented "502 Upstream Error" with LLM provider checks
- Documented "Channel connection failed" with channel test commands
- Documented "Sandbox execution timeout" with configuration fixes
- Documented "Config parse error" with validation commands

**2. Diagnostic Commands:**
- Documented `aisopod doctor` comprehensive health check
- Created targeted check options (`--check-config`, `--check-providers`, `--check-channels`, `--check-data`, `--check-sandbox`)
- Documented expected output format

**3. Log Analysis:**
- Documented log level configuration (trace, debug, info, warn, error)
- Created log level usage table
- Documented key log patterns for debugging
- Explained structured JSON logging with `AISOPOD_LOG_FORMAT=json`

**4. Channel-Specific Troubleshooting:**
- Documented Telegram troubleshooting (webhook, duplicate messages, media support)
- Documented Discord troubleshooting (token, permissions, rate limits)
- Documented WhatsApp troubleshooting (webhook verification, message delivery, templates)
- Documented Slack troubleshooting (Socket Mode, DMs, event subscriptions)

**5. Performance Tuning:**
- Documented gateway tuning options (workers, timeout, max_connections)
- Documented memory usage optimization
- Documented latency optimization strategies (streaming, provider selection)
- Documented throughput scaling approaches (load balancing, shared storage)

**6. Cargo Doc Generation:**
- Documented `cargo doc` generation command
- Explained local generation process
- Documented CI integration for automatic API documentation generation
- Included artifact combination steps

**7. Documentation Linking:**
- Updated SUMMARY.md with troubleshooting.md link

### Files Created/Modified
- docs/book/src/troubleshooting.md
- .github/workflows/ci.yml (docs job)

### Test Results
- mdbook build docs/book: PASSED
- cargo build: PASSED
- cargo doc --workspace --no-deps: PASSED

### Resolution Date
2026-02-27
