# Troubleshooting

This guide covers common errors, diagnostic tools, log analysis, and performance tuning for Aisopod.

## Common Errors

### `Error: Connection refused (port 3080)`

**Cause:** Gateway is not running or listening on a different port.

**Solution:**
```bash
# Check if gateway is running
aisopod gateway status

# Start if not running
aisopod gateway start

# Verify port
ss -tlnp | grep 3080
```

### `Error: 401 Unauthorized`

**Cause:** Missing or invalid authentication token.

**Solution:**
- Verify token in request: `Authorization: Bearer <token>`
- Check config: `aisopod config show | grep auth`
- If using env var: `echo $AISOPOD_AUTH_TOKEN`

### `Error: 502 Upstream Error`

**Cause:** LLM provider returned an error (rate limit, invalid key, outage).

**Solution:**
```bash
# Check provider status
aisopod doctor --check-providers

# Test API key directly
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $AISOPOD_OPENAI_API_KEY"
```

### `Error: Channel connection failed`

**Cause:** Invalid channel token or network issue.

**Solution:**
```bash
aisopod channel test telegram
```

### `Error: Sandbox execution timeout`

**Cause:** Code execution exceeded the configured timeout.

**Solution:**
- Increase timeout: `tools.sandbox.timeout = 60`
- Check if Docker daemon is running: `docker info`

### `Error: Config parse error`

**Cause:** Invalid TOML syntax in config file.

**Solution:**
```bash
aisopod config validate
```

### `Error: Rate limit exceeded`

**Cause:** LLM provider rate limit has been hit.

**Solution:**
```bash
# Check current rate limits
aisopod provider status

# Add retry logic with backoff in your clients
# Consider upgrading to a higher-tier plan
```

### `Error: Out of memory`

**Cause:** System or container memory limit exceeded.

**Solution:**
- Reduce session cache: `storage.session_cache_size = 100`
- Limit concurrent agents
- Increase system/container memory limits

## Diagnostic Commands

### `aisopod doctor`

Runs a comprehensive health check:

```bash
aisopod doctor
```

Output:
```
✅ Configuration: valid
✅ Authentication: token mode configured
✅ OpenAI API: connected (gpt-4o available)
✅ Anthropic API: connected (claude-3-opus available)
⚠️  Telegram channel: token valid, webhook not set
✅ Sandbox: Docker available, image pulled
✅ Storage: sessions.db accessible (142 sessions)
❌ Disk space: less than 100MB free — consider cleanup
```

### Targeted Checks

```bash
aisopod doctor --check-config       # Config file only
aisopod doctor --check-providers    # LLM provider connectivity
aisopod doctor --check-channels     # Channel connectivity
aisopod doctor --check-data         # Storage integrity
aisopod doctor --check-sandbox      # Sandbox runtime
```

### Additional Diagnostic Commands

```bash
# Show gateway status
aisopod gateway status

# Show provider status
aisopod provider status

# Show memory usage
aisopod gateway status --verbose

# Validate configuration
aisopod config validate

# Test channel connectivity
aisopod channel test <channel-name>
```

## Log Analysis

### Setting Log Level

```bash
# Environment variable
AISOPOD_LOG=debug aisopod gateway start

# Granular control
AISOPOD_LOG=aisopod_gateway=debug,aisopod_agent=info aisopod gateway start
```

### Log Levels

| Level   | Use Case                                    |
|---------|---------------------------------------------|
| `error` | Production — only errors                    |
| `warn`  | Production — errors + warnings              |
| `info`  | Default — request/response summaries        |
| `debug` | Development — detailed request flow         |
| `trace` | Debugging — full message payloads           |

### Key Log Patterns

```bash
# Find failed requests
grep "ERROR" /var/log/aisopod.log

# Trace a specific request
grep "request_id=abc123" /var/log/aisopod.log

# Monitor channel activity
AISOPOD_LOG=aisopod_channel=debug aisopod gateway start 2>&1 | grep channel
```

### Structured Logging

Aisopod outputs structured JSON logs when `AISOPOD_LOG_FORMAT=json`:

```json
{"timestamp":"2026-01-15T10:30:00Z","level":"INFO","target":"aisopod_gateway","message":"Request processed","request_id":"abc123","duration_ms":250}
```

### Common Log Patterns and Their Meanings

| Pattern | Meaning |
|---------|---------|
| `ERROR.*request_id=.*` | Failed request |
| `WARN.*timeout` | Timeout occurred |
| `DEBUG.*streaming` | Streaming response active |
| `INFO.*channel.*connected` | Channel successfully connected |
| `ERROR.*provider.*rate_limit` | Rate limit hit |

## Channel-Specific Troubleshooting

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

### General Channel Troubleshooting

```bash
# Test all channels
aisopod channel test all

# Test specific channel
aisopod channel test telegram
aisopod channel test discord
aisopod channel test whatsapp
aisopod channel test slack

# Check channel status
aisopod channel status
```

## Performance Tuning

### Gateway Tuning

```toml
[server]
workers = 8              # match CPU cores for I/O-bound workloads
request_timeout = 60     # increase for slow models
max_connections = 1024   # increase for high-traffic deployments
```

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

### Performance Monitoring Commands

```bash
# Check gateway stats
aisopod gateway status --verbose

# Monitor memory usage
ps aux | grep aisopod

# Track request latency
grep "duration_ms" /var/log/aisopod.log

# Check active connections
netstat -an | grep 3080 | grep ESTABLISHED | wc -l
```

## Rust API Documentation

Auto-generated Rust API documentation is available at `https://docs.aisopod.dev/api/`.

### Generate Locally

```bash
cargo doc --workspace --no-deps --document-private-items
open target/doc/aisopod/index.html
```

### API Documentation Structure

The generated documentation includes:

- All public crates in the workspace
- Module and trait documentation
- Struct and enum documentation with examples
- Method and function documentation
- Type definitions and generics

### Viewing API Docs

```bash
# Build API docs
cargo doc --workspace --no-deps

# Open in browser
xdg-open target/doc/aisopod/index.html
```

## Getting Help

If you're still experiencing issues after trying the troubleshooting steps above:

1. Check the [Getting Started](./getting-started.md) guide
2. Review the [Configuration](./configuration.md) reference
3. Visit the [Developer Guide](./developer-guide.md) for architecture details
4. Search existing [issues](https://github.com/aisopod/aisopod/issues)
5. Join our [community Discord](https://discord.gg/aisopod)
