//! Mock Twitch TMI (Twitch Messaging Interface) server for integration testing.
//!
//! This module provides a mock implementation of a Twitch TMI WebSocket server
//! that responds to standard Twitch IRC commands, allowing integration
//! tests to verify Twitch channel behavior without requiring a real Twitch connection.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::ops::Deref;
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

/// State for the Twitch mock server
#[derive(Clone, Default)]
pub struct MockTwitchState {
    /// Map of channels to their members
    pub channels: Arc<Mutex<HashMap<String, Vec<String>>>>,
    /// List of received messages
    pub received_messages: Arc<Mutex<Vec<TwitchMessage>>>,
    /// Number of JOIN commands received
    pub join_count: Arc<Mutex<usize>>,
    /// Number of PRIVMSG commands received
    pub privmsg_count: Arc<Mutex<usize>>,
    /// Number of PONG responses sent
    pub pong_count: Arc<Mutex<usize>>,
    /// Whispers sent
    pub whispers: Arc<Mutex<Vec<TwitchWhisper>>>,
}

impl MockTwitchState {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            received_messages: Arc::new(Mutex::new(Vec::new())),
            join_count: Arc::new(Mutex::new(0)),
            privmsg_count: Arc::new(Mutex::new(0)),
            pong_count: Arc::new(Mutex::new(0)),
            whispers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// A Twitch message
#[derive(Clone, Debug)]
pub struct TwitchMessage {
    pub channel: String,
    pub username: String,
    pub message: String,
    pub tags: TwitchMessageTags,
}

/// Twitch message tags
#[derive(Clone, Debug, Default)]
pub struct TwitchMessageTags {
    pub user_id: String,
    pub display_name: String,
    pub is_mod: bool,
    pub is_subscriber: bool,
    pub badges: Vec<String>,
}

/// A Twitch whisper
#[derive(Clone, Debug)]
pub struct TwitchWhisper {
    pub to_username: String,
    pub from_username: String,
    pub message: String,
}

/// A mock Twitch TMI WebSocket server
pub struct MockTwitchServer {
    /// The server address in format "host:port"
    pub addr: String,
    /// The state of the mock server
    pub state: MockTwitchState,
    /// The join handle for the server task
    _handle: tokio::task::JoinHandle<()>,
    /// Channel to signal server shutdown
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockTwitchServer {
    /// Starts a mock Twitch TMI server on a random port
    ///
    /// # Returns
    ///
    /// A tuple of (server address, MockTwitchServer instance)
    pub async fn start() -> (String, Self) {
        let state = MockTwitchState::new();
        let state_clone = state.clone();

        // Create a channel to signal shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let addr = listener.local_addr().expect("Failed to get local addr");
        let addr_str = format!("127.0.0.1:{}", addr.port());

        info!("Starting mock Twitch TMI server at {}", addr_str);

        let handle = tokio::spawn(async move {
            let mut shutdown_rx = Some(shutdown_rx);

            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((socket, _addr)) => {
                                info!("Twitch client connected from {}", _addr);
                                let state = state_clone.clone();
                                // Pass the socket directly to the handler
                                tokio::spawn(handle_twitch_client(socket, state));
                            }
                            Err(e) => {
                                warn!("Failed to accept Twitch connection: {}", e);
                                continue;
                            }
                        }
                    }
                    _ = shutdown_rx.as_mut().expect("Shutdown channel should not be None") => {
                        info!("Mock Twitch server shutdown signal received");
                        break;
                    }
                }
            }
        });

        let server = Self {
            addr: addr_str,
            state,
            _handle: handle,
            _shutdown_tx: Some(shutdown_tx),
        };

        let addr = server.addr.clone();
        (addr, server)
    }
}

/// Handle a Twitch client connection
async fn handle_twitch_client(
    mut socket: tokio::net::TcpStream,
    state: MockTwitchState,
) {
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpStream;
    
    // Split the stream into reader and writer halves
    let (reader, mut writer) = tokio::io::split(socket);
    
    // Spawn reader task
    let reader_state = state.clone();
    let reader_handle = tokio::spawn(async move {
        use tokio::io::{AsyncRead, AsyncReadExt};
        use tokio::io::AsyncBufReadExt;
        
        let mut reader = tokio::io::BufReader::new(reader);
        
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = String::from_utf8_lossy(&buf);
                    handle_twitch_command(&line.trim(), &reader_state).await;
                }
                Err(e) => {
                    warn!("Error reading Twitch command: {}", e);
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
            let _ = writer.write_all(b"PING :tmi.twitch.tv\r\n").await;
            let _ = writer.flush().await;
        }
    });

    // Wait for either task to complete
    let _ = tokio::join!(reader_handle, writer_handle);
}

/// Handle an incoming Twitch command
async fn handle_twitch_command(command: &str, state: &MockTwitchState) {
    info!("Received Twitch command: {}", command);

    // Parse the command
    let parts: Vec<&str> = command.splitn(2, ' ').collect();
    let cmd = parts.get(0).map(|s| s.to_uppercase()).unwrap_or_default();

    match cmd.as_str() {
        "CAP" => {
            // Twitch requires CAP REQUEST for tags
            info!("Client sent CAP command");
        }
        "NICK" => {
            // Parse nickname
            let nick = parts.get(1).unwrap_or(&"").trim();
            info!("Client set nickname: {}", nick);
        }
        "PASS" => {
            // OAuth token (ignored for testing)
            info!("Client sent PASS command");
        }
        "PING" => {
            // Reply with PONG
            let pong_text = "PONG :tmi.twitch.tv\r\n";
            info!("Sending PONG: {}", pong_text.trim());
            
            let mut count = state.pong_count.lock().unwrap();
            *count += 1;
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

            let msg = TwitchMessage {
                channel: target.to_string(),
                username: "testuser".to_string(),
                message: message.to_string(),
                tags: TwitchMessageTags {
                    user_id: "123456".to_string(),
                    display_name: "TestUser".to_string(),
                    is_mod: false,
                    is_subscriber: false,
                    badges: vec![],
                },
            };

            let mut messages = state.received_messages.lock().unwrap();
            messages.push(msg);

            let mut count = state.privmsg_count.lock().unwrap();
            *count += 1;

            info!("Received PRIVMSG to {}: {}", target, message);
        }
        "WHISPER" => {
            // Parse WHISPER <to_user> :<message>
            let rest = parts.get(1).unwrap_or(&"");
            let to_msg: Vec<&str> = rest.splitn(2, ':').collect();
            let to_user = to_msg.get(0).unwrap_or(&"testuser").trim();
            let message = to_msg.get(1).unwrap_or(&"").trim();

            let whisper = TwitchWhisper {
                to_username: to_user.to_string(),
                from_username: "testbot".to_string(),
                message: message.to_string(),
            };

            let mut whispers = state.whispers.lock().unwrap();
            whispers.push(whisper);

            info!("Received WHISPER to {}: {}", to_user, message);
        }
        "PART" => {
            // Parse PART command
            info!("Client parted from channel");
        }
        "USER" => {
            // Parse user command
            info!("Client sent USER command");
        }
        _ => {
            info!("Unknown Twitch command: {}", cmd);
        }
    }
}

impl Default for MockTwitchServer {
    fn default() -> Self {
        panic!("MockTwitchServer must be started with MockTwitchServer::start()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_twitch_server_starts() {
        let (_addr, server) = MockTwitchServer::start().await;
        assert!(server.addr.len() > 0);
        assert!(server.state.join_count.lock().unwrap().eq(&0));
    }

    #[tokio::test]
    async fn test_mock_twitch_join() {
        let (_addr, server) = MockTwitchServer::start().await;

        // Simulate a JOIN command
        let cmd = "JOIN #test\r\n";
        handle_twitch_command(cmd, &server.state).await;

        let channels = server.state.channels.lock().unwrap();
        assert!(channels.contains_key("#test"));
        assert!(channels.get("#test").unwrap().contains(&"testbot".to_string()));
    }

    #[tokio::test]
    async fn test_mock_twitch_privmsg() {
        let (_addr, server) = MockTwitchServer::start().await;

        // Simulate a PRIVMSG command
        let cmd = "PRIVMSG #test :Hello from Twitch\r\n";
        handle_twitch_command(cmd, &server.state).await;

        let messages = server.state.received_messages.lock().unwrap();
        assert!(!messages.is_empty());
        assert_eq!(messages[0].message, "Hello from Twitch");
    }
}
