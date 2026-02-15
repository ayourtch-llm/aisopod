# Issue 195: Write Security and Deployment Documentation

## Summary
Create comprehensive security and deployment documentation covering authentication modes, authorization scopes, sandbox configuration, production best practices, and step-by-step deployment guides for all supported platforms.

## Location
- Crate: N/A (documentation)
- File: `docs/book/src/security-deployment.md`

## Current Behavior
Security configuration and deployment procedures are undocumented. Users must guess at authentication options, and deploying to production requires reading source code and Docker files.

## Expected Behavior
A thorough security and deployment guide at `docs/book/src/security-deployment.md` that enables operators to securely configure and deploy Aisopod to any supported platform with confidence.

## Impact
Security misconfiguration is the most dangerous class of user error. Deployment complexity is the biggest barrier to production adoption. This documentation directly addresses both risks and is essential for any production use.

## Suggested Implementation

1. **Create** `docs/book/src/security-deployment.md` with the following sections:

2. **Authentication Modes section:**
   ```markdown
   ## Authentication

   ### Token Authentication (`mode = "token"`)
   \```toml
   [auth]
   mode = "token"
   token = "${AISOPOD_AUTH_TOKEN}"
   \```
   Clients authenticate with `Authorization: Bearer <token>`. Recommended
   for API-only access and single-user setups.

   ### Password Authentication (`mode = "password"`)
   \```toml
   [auth]
   mode = "password"
   password_hash = "$argon2id$..."  # generate with: aisopod auth hash-password
   \```
   Clients authenticate via a login endpoint that issues session tokens.
   Recommended for multi-user deployments with a web UI.

   ### Device Authentication (`mode = "device"`)
   \```toml
   [auth]
   mode = "device"
   \```
   OAuth2 device authorization flow. User authorizes via browser.
   Recommended for CLI clients and IoT devices.

   ### No Authentication (`mode = "none"`)
   \```toml
   [auth]
   mode = "none"
   \```
   ⚠️ **Only use for local development.** All requests are accepted without
   authentication.
   ```

3. **Authorization Scopes section:**
   ```markdown
   ## Authorization Scopes

   | Scope             | Description                              |
   |-------------------|------------------------------------------|
   | `chat:read`       | Read chat messages and history            |
   | `chat:write`      | Send messages to agents                   |
   | `agent:read`      | View agent configuration                  |
   | `agent:manage`    | Create, update, delete agents             |
   | `channel:read`    | View channel configuration                |
   | `channel:manage`  | Add, remove, configure channels           |
   | `config:read`     | View server configuration                 |
   | `config:write`    | Modify server configuration               |
   | `admin`           | Full administrative access                |

   Assign scopes to tokens:
   \```bash
   aisopod auth create-token --scopes chat:read,chat:write --name "api-client"
   \```
   ```

4. **Sandbox Configuration section:**
   ```markdown
   ## Sandbox Configuration

   Code execution skills run in a sandboxed environment.

   \```toml
   [tools.sandbox]
   enabled = true
   runtime = "docker"        # docker | wasm | none
   image = "aisopod/sandbox:latest"
   timeout = 30              # seconds
   memory_limit = "256m"
   cpu_limit = "0.5"
   network = false           # disable network access in sandbox
   allowed_languages = ["python", "javascript", "bash"]
   \```

   **Best practices:**
   - Always set `network = false` unless explicitly needed
   - Set conservative `timeout` and `memory_limit`
   - Use specific `allowed_languages` rather than allowing all
   ```

5. **Production Best Practices section:**
   ```markdown
   ## Production Best Practices

   - **Always enable authentication** — never run `mode = "none"` in production
   - **Use environment variables for secrets** — never hardcode tokens in config files
   - **Enable TLS** — use a reverse proxy (nginx, Caddy) for HTTPS termination
   - **Set resource limits** — configure `workers`, `timeout`, `memory_limit`
   - **Enable structured logging** — set `AISOPOD_LOG=info` and pipe to log aggregator
   - **Monitor health endpoint** — poll `/health` for uptime monitoring
   - **Rotate tokens regularly** — use `aisopod auth rotate-token`
   - **Back up session data** — back up `sessions.db` on a schedule
   - **Restrict network in sandbox** — set `sandbox.network = false`
   ```

6. **Deployment Guides section:**
   ```markdown
   ## Deployment Guides

   ### Docker
   \```bash
   docker run -d \
     --name aisopod \
     -p 3080:3080 \
     -v /opt/aisopod/config:/config \
     -v /opt/aisopod/data:/data \
     -e AISOPOD_AUTH_TOKEN=your-token \
     -e AISOPOD_OPENAI_API_KEY=sk-... \
     --restart unless-stopped \
     ghcr.io/aisopod/aisopod:latest
   \```

   ### Docker Compose
   \```yaml
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
   \```

   ### Fly.io
   \```bash
   fly launch --image ghcr.io/aisopod/aisopod:latest
   fly secrets set AISOPOD_AUTH_TOKEN=your-token
   fly secrets set AISOPOD_OPENAI_API_KEY=sk-...
   fly deploy
   \```

   ### Render
   1. Create a new Web Service
   2. Set Docker image: `ghcr.io/aisopod/aisopod:latest`
   3. Set environment variables in Render dashboard
   4. Deploy

   ### VPS (Ubuntu/Debian)
   \```bash
   # Install binary
   curl -fsSL https://aisopod.dev/install.sh | sh

   # Create system user
   sudo useradd -r -s /bin/false aisopod

   # Create directories
   sudo mkdir -p /etc/aisopod /var/lib/aisopod
   sudo chown aisopod:aisopod /var/lib/aisopod

   # Copy config
   sudo cp config.toml /etc/aisopod/config.toml

   # Set up systemd (see below)
   \```

   ### Raspberry Pi
   \```bash
   # Cross-compile for ARM
   cargo build --release --target aarch64-unknown-linux-gnu

   # Or use Docker on Pi
   docker run -d --name aisopod -p 3080:3080 \
     ghcr.io/aisopod/aisopod:latest
   \```

   ### systemd Service
   \```ini
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

   [Install]
   WantedBy=multi-user.target
   \```

   \```bash
   sudo cp aisopod.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now aisopod
   \```

   ### launchctl (macOS)
   \```xml
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
     </array>
     <key>RunAtLoad</key>
     <true/>
     <key>KeepAlive</key>
     <true/>
   </dict>
   </plist>
   \```

   \```bash
   cp dev.aisopod.gateway.plist ~/Library/LaunchAgents/
   launchctl load ~/Library/LaunchAgents/dev.aisopod.gateway.plist
   \```
   ```

7. **Reverse Proxy Configuration section:**
   ```markdown
   ## Reverse Proxy

   ### nginx
   \```nginx
   server {
       listen 443 ssl http2;
       server_name aisopod.example.com;

       ssl_certificate /etc/letsencrypt/live/aisopod.example.com/fullchain.pem;
       ssl_certificate_key /etc/letsencrypt/live/aisopod.example.com/privkey.pem;

       location / {
           proxy_pass http://127.0.0.1:3080;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
           proxy_set_header X-Forwarded-Proto $scheme;
       }

       location /ws {
           proxy_pass http://127.0.0.1:3080/ws;
           proxy_http_version 1.1;
           proxy_set_header Upgrade $http_upgrade;
           proxy_set_header Connection "upgrade";
       }
   }
   \```

   ### Caddy
   \```
   aisopod.example.com {
       reverse_proxy localhost:3080
   }
   \```
   Caddy automatically provisions TLS certificates via Let's Encrypt.
   ```

8. **Update `SUMMARY.md`** to link to this page.

## Dependencies
- Issue 187 (documentation infrastructure)
- Issue 152 (security tests validate auth modes and scopes)
- Issue 161 (deployment tests validate platform-specific configurations)

## Acceptance Criteria
- [ ] `docs/book/src/security-deployment.md` exists and is linked from `SUMMARY.md`
- [ ] All authentication modes documented (token, password, device, none)
- [ ] Authorization scopes reference table is complete
- [ ] Sandbox configuration documented with security best practices
- [ ] Production best practices checklist included
- [ ] Deployment guides cover: Docker, Docker Compose, Fly.io, Render, VPS, Raspberry Pi
- [ ] systemd and launchctl service files are provided
- [ ] Reverse proxy configurations provided for nginx and Caddy (including WebSocket)
- [ ] `mdbook build` succeeds with this page included

---
*Created: 2026-02-15*
