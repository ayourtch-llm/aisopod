# Security & Deployment

## Authentication

Aisopod supports four authentication modes that can be configured in the `config.toml` file.

### Token Authentication (`mode = "token"`)

Token authentication is the simplest and most secure mode for API-only access and single-user setups. Clients authenticate using a Bearer token in the Authorization header.

```toml
[auth]
mode = "token"
token = "${AISOPOD_AUTH_TOKEN}"
```

 Clients authenticate with `Authorization: Bearer <token>`.

**Use cases:**
- API-only access
- Single-user deployments
- Service-to-service communication

### Password Authentication (`mode = "password"`)

Password authentication provides a login-based authentication flow with session tokens. This mode is recommended for multi-user deployments with a web UI.

```toml
[auth]
mode = "password"
password_hash = "$argon2id$"  # Generate with: aisopod auth hash-password
```

Clients authenticate via a login endpoint that issues session tokens.

**Use cases:**
- Multi-user deployments
- Web UI access
- Password-based authentication flow

**Generating password hashes:**
```bash
aisopod auth hash-password
```

### Device Authentication (`mode = "device"`)

Device authentication uses the OAuth2 device authorization flow. Users authorize devices via a browser-based flow, making it ideal for CLI clients and IoT devices that cannot easily handle OAuth redirects.

```toml
[auth]
mode = "device"
```

**Use cases:**
- CLI clients
- IoT devices
- Devices without easy browser access

### No Authentication (`mode = "none"`)

```toml
[auth]
mode = "none"
```

⚠️ **Only use for local development.** All requests are accepted without authentication.

**Use cases:**
- Local development only
- Testing environments
- ⚠️ Never use in production

## Authorization Scopes

Authorization scopes allow fine-grained control over API access. Each token or session can be assigned specific scopes limiting what actions are permitted.

| Scope             | Description                              |
|-------------------|------------------------------------------|
| `chat:read`       | Read chat messages and history           |
| `chat:write`      | Send messages to agents                  |
| `agent:read`      | View agent configuration                 |
| `agent:manage`    | Create, update, delete agents            |
| `channel:read`    | View channel configuration               |
| `channel:manage`  | Add, remove, configure channels          |
| `config:read`     | View server configuration                |
| `config:write`    | Modify server configuration              |
| `admin`           | Full administrative access               |

### Creating Tokens with Specific Scopes

Assign scopes to tokens using the CLI:

```bash
aisopod auth create-token --scopes chat:read,chat:write --name "api-client"
```

### Default Scopes

- **System tokens** (internal): All scopes
- **User tokens** (created via CLI): Admin scope by default
- **Session tokens**: Inherited from user's scopes

## Sandbox Configuration

Code execution skills run in a sandboxed environment to prevent malicious code from affecting the host system.

### Configuration

```toml
[tools.sandbox]
enabled = true
runtime = "docker"        # docker | wasm | none
image = "aisopod/sandbox:latest"
timeout = 30              # seconds
memory_limit = "256m"
cpu_limit = "0.5"
network = false           # disable network access in sandbox
allowed_languages = ["python", "javascript", "bash"]
```

### Sandbox Runtime Options

- **docker**: Runs code in Docker containers (recommended for production)
- **wasm**: Runs code in WebAssembly (sandboxed but limited)
- **none**: No sandboxing - code runs directly on host (dangerous!)

### Security Best Practices

1. **Always set `network = false`** unless explicitly needed for the skill
2. Set conservative `timeout` (default 30s) and `memory_limit` values
3. Use specific `allowed_languages` rather than allowing all languages
4. Keep the sandbox image updated with security patches
5. Monitor sandbox execution logs for suspicious activity
6. Consider setting `cpu_limit` to prevent resource exhaustion
7. Never run untrusted code with `network = true`

### Example: Strict Production Sandbox

```toml
[tools.sandbox]
enabled = true
runtime = "docker"
image = "aisopod/sandbox:latest"
timeout = 30
memory_limit = "128m"
cpu_limit = "0.25"
network = false
allowed_languages = ["python"]
```

## Production Best Practices

Follow this 10-item checklist before deploying Aisopod to production:

1. **Always enable authentication** — never run `mode = "none"` in production
2. **Use environment variables for secrets** — never hardcode tokens in config files; use `${VARIABLE}` syntax
3. **Enable TLS** — use a reverse proxy (nginx, Caddy) for HTTPS termination
4. **Set resource limits** — configure `workers`, `timeout`, `memory_limit` based on expected load
5. **Enable structured logging** — set `AISOPOD_LOG=info` and pipe logs to a log aggregator
6. **Monitor health endpoint** — poll `/health` for uptime monitoring and alerting
7. **Rotate tokens regularly** — use `aisopod auth rotate-token` on a scheduled basis
8. **Back up session data** — back up `sessions.db` on a regular schedule
9. **Restrict network in sandbox** — set `sandbox.network = false` unless a skill requires network access
10. **Use systemd or similar** — deploy as a managed service with automatic restart on failure

### Additional Security Recommendations

- **Rate limiting**: Consider adding rate limiting at the reverse proxy level
- **IP allowlisting**: For admin endpoints, consider restricting access by IP
- **Audit logging**: Enable detailed logging for security-critical operations
- **Secret rotation**: Rotate API keys and tokens periodically
- **Security updates**: Subscribe to security advisories for Aisopod

## Deployment Guides

### Docker

Run Aisopod in a Docker container:

```bash
docker run -d \
  --name aisopod \
  -p 3080:3080 \
  -v /opt/aisopod/config:/config \
  -v /opt/aisopod/data:/data \
  -e AISOPOD_AUTH_TOKEN=your-token \
  -e AISOPOD_OPENAI_API_KEY=sk-... \
  --restart unless-stopped \
  ghcr.io/aisopod/aisopod:latest
```

**Explanation:**
- `-d`: Run in detached mode
- `-p 3080:3080`: Map container port 3080 to host port 3080
- `-v /opt/aisopod/config:/config`: Mount config directory
- `-v /opt/aisopod/data:/data`: Mount data directory for persistent storage
- `-e`: Set environment variables for secrets
- `--restart unless-stopped`: Auto-restart on failure

### Docker Compose

Use Docker Compose for more complex deployments:

```yaml
version: "3.8"
services:
  aisopod:
    image: ghcr.io/aisopod/aisopod:latest
    ports:
      - "3080:3080"
    volumes:
      - ./config:/config
      - aisopod-data:/data
    environment:
      - AISOPOD_AUTH_TOKEN=${AISOPOD_AUTH_TOKEN}
      - AISOPOD_OPENAI_API_KEY=${AISOPOD_OPENAI_API_KEY}
    restart: unless-stopped
volumes:
  aisopod-data:
```

**To use:**
1. Create `.env` file with secrets
2. Run `docker compose up -d`

### Fly.io

Deploy to Fly.io for managed hosting:

```bash
fly launch --image ghcr.io/aisopod/aisopod:latest
fly secrets set AISOPOD_AUTH_TOKEN=your-token
fly secrets set AISOPOD_OPENAI_API_KEY=sk-...
fly deploy
```

**Additional Fly.io configuration (`fly.toml`):**
```toml
[http_service]
internal_port = 3080
force_https = true

[[http_service.checks]]
grace_period = "10s"
interval = "15s"
method = "GET"
path = "/health"
timeout = "5s"
```

### Render

Deploy to Render for managed hosting:

1. Create a new Web Service in the Render dashboard
2. Set Docker image: `ghcr.io/aisopod/aisopod:latest`
3. Set environment variables in the Render dashboard:
   - `AISOPOD_AUTH_TOKEN`
   - `AISOPOD_OPENAI_API_KEY`
4. Set health check path: `/health`
5. Deploy

### VPS (Ubuntu/Debian)

Deploy to a VPS running Ubuntu or Debian:

```bash
# Install binary
curl -fsSL https://aisopod.dev/install.sh | sh

# Create system user
sudo useradd -r -s /bin/false aisopod

# Create directories
sudo mkdir -p /etc/aisopod /var/lib/aisopod
sudo chown aisopod:aisopod /var/lib/aisopod

# Copy config
sudo cp config.toml /etc/aisopod/config.toml
sudo chown aisopod:aisopod /etc/aisopod/config.toml

# Set up systemd (see below)
```

### Raspberry Pi

Deploy to a Raspberry Pi using Docker or cross-compilation:

**Using Docker:**
```bash
docker run -d --name aisopod -p 3080:3080 \
  -v /opt/aisopod/config:/config \
  -v /opt/aisopod/data:/data \
  -e AISOPOD_AUTH_TOKEN=${AISOPOD_AUTH_TOKEN} \
  -e AISOPOD_OPENAI_API_KEY=${AISOPOD_OPENAI_API_KEY} \
  ghcr.io/aisopod/aisopod:latest
```

**Using cross-compilation:**
```bash
# Cross-compile for ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Transfer binary to Raspberry Pi
scp target/aarch64-unknown-linux-gnu/release/aisopod pi@raspberrypi:/usr/local/bin/

# Set up systemd service
```

### systemd Service

Create a systemd service file for Linux systems:

**`/etc/systemd/system/aisopod.service`:**
```ini
[Unit]
Description=Aisopod AI Gateway
After=network.target

[Service]
Type=simple
User=aisopod
ExecStart=/usr/local/bin/aisopod gateway start --config /etc/aisopod/config.toml
Restart=on-failure
RestartSec=5
Environment=AISOPOD_LOG=info
WorkingDirectory=/var/lib/aisopod

[Install]
WantedBy=multi-user.target
```

**Enable and start the service:**
```bash
sudo cp aisopod.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now aisopod
```

**Manage the service:**
```bash
# Check status
sudo systemctl status aisopod

# View logs
sudo journalctl -u aisopod -f

# Restart
sudo systemctl restart aisopod

# Stop
sudo systemctl stop aisopod
```

### launchctl (macOS)

Create a launchd plist for macOS:

**`~/Library/LaunchAgents/dev.aisopod.gateway.plist`:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.aisopod.gateway</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/local/bin/aisopod</string>
    <string>gateway</string>
    <string>start</string>
    <string>--config</string>
    <string>/etc/aisopod/config.toml</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardErrorPath</key>
  <string>/var/log/aisopod.log</string>
  <key>StandardOutPath</key>
  <string>/var/log/aisopod.log</string>
</dict>
</plist>
```

**Load the service:**
```bash
cp dev.aisopod.gateway.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/dev.aisopod.gateway.plist
```

**Manage the service:**
```bash
# Stop
launchctl unload ~/Library/LaunchAgents/dev.aisopod.gateway.plist

# Start
launchctl load ~/Library/LaunchAgents/dev.aisopod.gateway.plist
```

## Reverse Proxy Configuration

### nginx

Configure nginx as a reverse proxy with TLS termination and WebSocket support:

```nginx
upstream aisopod {
    server 127.0.0.1:3080;
}

server {
    listen 80;
    server_name aisopod.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name aisopod.example.com;

    ssl_certificate /etc/letsencrypt/live/aisopod.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/aisopod.example.com/privkey.pem;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Logging
    access_log /var/log/nginx/aisopod-access.log;
    error_log /var/log/nginx/aisopod-error.log;

    location / {
        proxy_pass http://aisopod;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location /ws {
        proxy_pass http://aisopod/ws;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # Health check endpoint (optional: bypass auth)
    location /health {
        proxy_pass http://aisopod/health;
        proxy_set_header Host $host;
    }
}
```

**Generate SSL certificates with Let's Encrypt:**
```bash
sudo certbot certonly --nginx -d aisopod.example.com
```

### Caddy

Caddy provides automatic TLS certificate management with Let's Encrypt:

```caddy
aisopod.example.com {
    reverse_proxy localhost:3080
}
```

Caddy automatically:
- Provisions and renews TLS certificates
- Redirects HTTP to HTTPS
- Handles WebSocket upgrades automatically
- Provides forward secrecy and modern cipher suites

**With additional configuration:**
```caddy
aisopod.example.com {
    # Reverse proxy
    reverse_proxy localhost:3080

    # Security headers
    header {
        X-Frame-Options "SAMEORIGIN"
        X-Content-Type-Options "nosniff"
        X-XSS-Protection "1; mode=block"
    }

    # Logging
    log {
        output file /var/log/caddy/aisopod.log
    }
}
```

**Install Caddy on Ubuntu/Debian:**
```bash
sudo apt install -y caddy
```

**Systemd service is provided automatically.**

## API Endpoints

### Authentication Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/auth/login` | Login with password (returns session token) |
| POST | `/auth/logout` | Logout current session |
| POST | `/auth/refresh` | Refresh session token |
| POST | `/auth/create-token` | Create a new API token (admin only) |
| POST | `/auth/rotate-token` | Rotate system tokens (admin only) |

### Health Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check (no auth required) |
| GET | `/ready` | Readiness check (no auth required) |

### WebSocket Endpoints

| Endpoint | Description |
|----------|-------------|
| `/ws` | WebSocket connection for real-time chat |

## Troubleshooting

### Common Issues

**Authentication failing with token:**
- Verify `AISOPOD_AUTH_TOKEN` environment variable is set
- Check that the token format matches expected format
- Review logs with `journalctl -u aisopod -f`

**WebSocket connections failing:**
- Ensure reverse proxy is configured for WebSocket upgrades
- Check `Upgrade` and `Connection` headers are passed
- Verify `/ws` location is properly configured in nginx

**Sandbox not executing code:**
- Verify Docker is running: `systemctl status docker`
- Check sandbox image exists: `docker images aisopod/sandbox`
- Review sandbox logs: `docker logs aisopod-sandbox`

**Service won't start:**
- Verify config file syntax: `aisopod gateway validate-config`
- Check file permissions on config and data directories
- Review service logs for specific error messages
