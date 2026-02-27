//! Mock IRC server for integration testing.
//!
//! This module provides a mock implementation of an IRC server
//! that responds to standard IRC commands, allowing integration
//! tests to verify IRC channel behavior without requiring a real IRC server.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tracing::{info, warn};

/// State for the IRC mock server
#[derive(Clone, Default)]
pub struct MockIrcState {
    /// Map of channels to their members
    pub channels: Arc<Mutex<HashMap<String, Vec<String>>>>,
    /// List of received messages
    pub received_messages: Arc<Mutex<Vec<IrcMessage>>>,
    /// Number of JOIN commands received
    pub join_count: Arc<Mutex<usize>>,
    /// Number of PRIVMSG commands received
    pub privmsg_count: Arc<Mutex<usize>>,
    /// NickServ authentication requests
    pub nickserv_auth_requests: Arc<Mutex<usize>>,
    /// Pending pings
    pub pending_pings: Arc<Mutex<Vec<String>>>,
}

impl MockIrcState {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            received_messages: Arc::new(Mutex::new(Vec::new())),
            join_count: Arc::new(Mutex::new(0)),
            privmsg_count: Arc::new(Mutex::new(0)),
            nickserv_auth_requests: Arc::new(Mutex::new(0)),
            pending_pings: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// An IRC message
#[derive(Clone, Debug)]
pub struct IrcMessage {
    pub command: String,
    pub params: Vec<String>,
    pub text: String,
}

/// A mock IRC server
pub struct MockIrcServer {
    /// The server address in format "host:port"
    pub addr: String,
    /// The state of the mock server
    pub state: MockIrcState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockIrcServer {
    /// Starts a mock IRC server on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server address, MockIrcServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockIrcState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let addr_str = format!("127.0.0.1:{}", addr.port());

        info!("Starting mock IRC server at {}", addr_str);

        let addr_for_closure = addr_str.clone();
        let handle = tokio::spawn(async move {
            let mut shutdown_rx = Some(shutdown_rx);

            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((socket, _addr)) => {
                                info!("IRC client connected from {}", _addr);
                                let state = state_clone.clone();
                                tokio::spawn(handle_irc_client(socket, state));
                            }
                            Err(e) => {
                                warn!("Failed to accept IRC connection: {}", e);
                                continue;
                            }
                        }
                    }
                    _ = shutdown_rx.as_mut().expect("Shutdown channel should not be None") => {
                        info!("Mock IRC server shutdown signal received");
                        break;
                    }
                }
            }
        });

        let server = Self {
            addr: addr_str.clone(),
            state,
            _handle: handle,
            _shutdown_tx: Some(shutdown_tx),
        };

        (addr_for_closure, server)
    }
}

/// Handle an IRC client connection
async fn handle_irc_client<S: AsyncRead + AsyncWrite + Unpin + 'static + Send>(mut socket: S, state: MockIrcState) {
    let (reader, mut writer) = tokio::io::split(socket);

    // Spawn reader task
    let reader_state = state.clone();
    let reader_handle = tokio::spawn(async move {
        let mut buf = Vec::new();
        let mut reader = tokio::io::BufReader::new(reader);

        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = String::from_utf8_lossy(&buf);
                    handle_irc_command(&line.trim(), &reader_state).await;
                }
                Err(e) => {
                    warn!("Error reading IRC command: {}", e);
                    break;
                }
            }
        }
    });

    // Spawn writer task to send periodic PINGs
    let writer_state = state.clone();
    let writer_handle = tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let _ = writer.write_all(b"PING :localhost\r\n").await;
            let _ = writer.flush().await;
        }
    });

    // Wait for either task to complete
    let _ = tokio::join!(reader_handle, writer_handle);
}

/// Handle an incoming IRC command
async fn handle_irc_command(command: &str, state: &MockIrcState) {
    info!("Received IRC command: {}", command);

    // Parse the command
    let parts: Vec<&str> = command.splitn(2, ' ').collect();
    let cmd = parts.get(0).map(|s| s.to_uppercase()).unwrap_or_default();

    match cmd.as_str() {
        "NICK" => {
            // Parse nickname
            let nick = parts.get(1).unwrap_or(&"").trim();
            info!("Client set nickname: {}", nick);
        }
        "USER" => {
            // Parse user command (USER <user> <mode> <unused> <realname>)
            info!("Client sent USER command");
        }
        "PING" => {
            // Reply with PONG
            let params = parts.get(1).unwrap_or(&":localhost");
            let _ = tokio::net::TcpStream::connect("127.0.0.1:0").await; // dummy to avoid unused
            // In real implementation, send PONG back
        }
        "PONG" => {
            // Acknowledge PONG
            let _ = state.pending_pings.lock().unwrap().pop();
        }
        "JOIN" => {
            // Parse channel
            let channel = parts.get(1).unwrap_or(&"#test").trim();
            let mut channels = state.channels.lock().unwrap();
            let members = channels.entry(channel.to_string()).or_insert_with(Vec::new);
            if !members.contains(&"testbot".to_string()) {
                members.push("testbot".to_string());
            }
            let mut count = state.join_count.lock().unwrap();
            *count += 1;

            info!("Client joined channel: {}", channel);
        }
        "PRIVMSG" => {
            // Parse PRIVMSG <target> :<message>
            let rest = parts.get(1).unwrap_or(&"");
            let target_msg: Vec<&str> = rest.splitn(2, ':').collect();
            let target = target_msg.get(0).unwrap_or(&"#test");
            let message = target_msg.get(1).unwrap_or(&"").trim();

            let msg = IrcMessage {
                command: "PRIVMSG".to_string(),
                params: vec![target.to_string()],
                text: message.to_string(),
            };

            let mut messages = state.received_messages.lock().unwrap();
            messages.push(msg);

            let mut count = state.privmsg_count.lock().unwrap();
            *count += 1;

            info!("Received PRIVMSG to {}: {}", target, message);
        }
        "MODE" => {
            // Handle mode commands
            info!("Client sent MODE command");
        }
        "QUIT" => {
            info!("Client quit");
        }
        "NICKSERV" => {
            // Handle NickServ authentication
            let rest = parts.get(1).unwrap_or(&"");
            let mut guard = state.nickserv_auth_requests.lock().unwrap();
            *guard += 1;
            info!("NickServ auth request: {}", rest);
        }
        _ => {
            info!("Unknown IRC command: {}", cmd);
        }
    }
}

impl Default for MockIrcServer {
    fn default() -> Self {
        panic!("MockIrcServer must be started with MockIrcServer::start()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_irc_server_starts() {
        let (_addr, server) = MockIrcServer::start().await;
        assert!(server.addr.len() > 0);
        assert!(server.state.join_count.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_irc_join() {
        let (_addr, server) = MockIrcServer::start().await;

        // Simulate a JOIN command
        let cmd = "JOIN #test\r\n";
        handle_irc_command(cmd, &server.state).await;

        let channels = server.state.channels.lock().unwrap();
        assert!(channels.contains_key("#test"));
        assert!(channels.get("#test").unwrap().contains(&"testbot".to_string()));
    }
}
