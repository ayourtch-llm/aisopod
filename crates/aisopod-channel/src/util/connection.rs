//! Connection state management and reconnection logic.
//!
//! This module provides utilities for managing connection states across
//! different channel implementations, including automatic reconnection
//! with exponential backoff.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tracing::{error, info, warn};

/// Represents the current state of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Currently attempting to connect
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection lost, attempting to reconnect
    Reconnecting,
    /// Connection failed (requires manual intervention)
    Failed,
}

impl ConnectionState {
    /// Check if the connection is active.
    pub fn is_active(&self) -> bool {
        matches!(self, ConnectionState::Connected)
    }

    /// Check if the connection is in progress.
    pub fn is_connecting(&self) -> bool {
        matches!(self, ConnectionState::Connecting | ConnectionState::Reconnecting)
    }

    /// Check if the connection is lost.
    pub fn is_lost(&self) -> bool {
        matches!(self, ConnectionState::Disconnected | ConnectionState::Connecting | ConnectionState::Reconnecting | ConnectionState::Failed)
    }
}

/// Statistics about connection history.
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Total number of reconnection attempts
    pub reconnection_attempts: u64,
    /// Total uptime in seconds
    pub total_uptime_seconds: u64,
    /// Timestamp of last connection start
    pub last_connection_start: Option<Instant>,
    /// Timestamp of last successful connection
    pub last_connected: Option<Instant>,
    /// Timestamp of last disconnection
    pub last_disconnected: Option<Instant>,
}

/// Configuration for connection management.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Initial delay before first reconnection attempt
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_delay: Duration,
    /// Backoff multiplier (delay is multiplied by this each attempt)
    pub backoff_multiplier: f64,
    /// Maximum number of reconnection attempts (0 = unlimited)
    pub max_reconnection_attempts: u32,
    /// Whether to use jitter to prevent thundering herd
    pub use_jitter: bool,
    /// Jitter factor (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300), // 5 minutes
            backoff_multiplier: 2.0,
            max_reconnection_attempts: 0, // unlimited
            use_jitter: true,
            jitter_factor: 0.1, // 10% jitter
        }
    }
}

/// A connection manager that handles state tracking and reconnection.
///
/// This struct maintains connection state and provides utilities for
/// establishing and maintaining connections with automatic reconnection
/// using exponential backoff.
pub struct ConnectionManager {
    /// Current connection state
    state: watch::Sender<ConnectionState>,
    /// Last observed state (for reading)
    last_state: watch::Receiver<ConnectionState>,
    /// Connection configuration
    config: ConnectionConfig,
    /// Connection statistics
    stats: std::sync::Mutex<ConnectionStats>,
    /// Current backoff delay
    current_delay: std::sync::Mutex<Duration>,
    /// Number of failed attempts
    failed_attempts: std::sync::Mutex<u32>,
}

impl ConnectionManager {
    /// Create a new connection manager with default configuration.
    pub fn new() -> Self {
        Self::with_config(ConnectionConfig::default())
    }

    /// Create a new connection manager with custom configuration.
    pub fn with_config(config: ConnectionConfig) -> Self {
        let (state_tx, state_rx) = watch::channel(ConnectionState::Disconnected);
        Self {
            state: state_tx,
            last_state: state_rx,
            config: config.clone(),
            stats: std::sync::Mutex::new(ConnectionStats::default()),
            current_delay: std::sync::Mutex::new(config.initial_delay),
            failed_attempts: std::sync::Mutex::new(0),
        }
    }

    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        *self.last_state.borrow()
    }

    /// Watch for connection state changes.
    ///
    /// This returns a new watch receiver that can be used to monitor
    /// state changes without blocking.
    pub fn watch(&self) -> watch::Receiver<ConnectionState> {
        self.last_state.clone()
    }

    /// Update the connection state.
    fn set_state(&self, new_state: ConnectionState) {
        let _ = self.state.send(new_state);
    }

    /// Update connection statistics.
    fn update_stats(&self, update: impl FnOnce(&mut ConnectionStats)) {
        let mut stats = self.stats.lock().unwrap();
        update(&mut stats);
    }

    /// Get connection statistics.
    pub fn stats(&self) -> ConnectionStats {
        self.stats.lock().unwrap().clone()
    }

    /// Record a successful connection.
    pub fn record_connect(&self) {
        let now = Instant::now();
        self.set_state(ConnectionState::Connected);
        self.update_stats(|s| {
            s.last_connection_start = Some(now);
            s.last_connected = Some(now);
        });
    }

    /// Record a disconnection.
    pub fn record_disconnect(&self) {
        let now = Instant::now();
        self.set_state(ConnectionState::Disconnected);
        self.update_stats(|s| {
            if let Some(start) = s.last_connection_start {
                s.total_uptime_seconds += (now - start).as_secs();
            }
            s.last_disconnected = Some(now);
        });
    }

    /// Record a failed connection attempt.
    pub fn record_connect_failed(&self) {
        let now = Instant::now();
        self.set_state(ConnectionState::Failed);
        self.update_stats(|s| {
            if let Some(start) = s.last_connection_start {
                s.total_uptime_seconds += (now - start).as_secs();
            }
            s.last_disconnected = Some(now);
        });
        *self.failed_attempts.lock().unwrap() += 1;
    }

    /// Record a reconnection attempt.
    pub fn record_reconnect_attempt(&self) {
        let now = Instant::now();
        self.set_state(ConnectionState::Reconnecting);
        self.update_stats(|s| {
            s.last_connection_start = Some(now);
            s.reconnection_attempts += 1;
        });
        *self.failed_attempts.lock().unwrap() += 1;
    }

    /// Get the next reconnection delay with exponential backoff and jitter.
    pub fn next_delay(&self) -> Duration {
        let mut current_delay = self.current_delay.lock().unwrap();
        let failed_attempts = self.failed_attempts.lock().unwrap();

        // Calculate delay with exponential backoff
        let delay = current_delay.as_secs_f64() * self.config.backoff_multiplier;
        let delay = Duration::from_secs(delay.max(1.0).floor() as u64);

        // Cap at max delay
        let new_delay = delay.min(self.config.max_delay);

        // Store the new delay for next time
        *current_delay = new_delay;

        // Apply jitter if configured, then cap at max delay
        if self.config.use_jitter {
            let jitter_range = (new_delay.as_secs_f64() * self.config.jitter_factor) as u64;
            let jitter = (rand::random::<u64>() % (jitter_range * 2 + 1)).saturating_sub(jitter_range);
            let delayed_with_jitter = Duration::from_secs(new_delay.as_secs().saturating_add(jitter));
            delayed_with_jitter.min(self.config.max_delay)
        } else {
            new_delay
        }
    }

    /// Reset backoff delay (called on successful connection).
    pub fn reset_backoff(&self) {
        *self.current_delay.lock().unwrap() = self.config.initial_delay;
        *self.failed_attempts.lock().unwrap() = 0;
    }

    /// Check if more reconnection attempts are allowed.
    pub fn can_retry(&self) -> bool {
        if self.config.max_reconnection_attempts == 0 {
            return true;
        }
        *self.failed_attempts.lock().unwrap() < self.config.max_reconnection_attempts
    }

    /// Get the number of failed attempts.
    pub fn failed_attempts(&self) -> u32 {
        *self.failed_attempts.lock().unwrap()
    }

    /// Maintain a connection by continuously attempting to reconnect.
    ///
    /// This async function continuously attempts to maintain a connection
    /// by calling the provided closure to establish a connection and
    /// automatically reconnecting on failure with exponential backoff.
    ///
    /// # Arguments
    ///
    /// * `connect` - A closure that establishes a connection. Should return
    ///   `Ok(())` on success or an error on failure.
    ///
    /// # Returns
    ///
    /// Returns when the connection is successfully established or when
    /// the maximum number of reconnection attempts is reached.
    pub async fn maintain_connection<F, Fut>(&self, connect: F) -> Result<(), String>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        loop {
            // Try to connect
            match connect().await {
                Ok(()) => {
                    self.record_connect();
                    self.reset_backoff();
                    info!("Successfully connected");
                    return Ok(());
                }
                Err(e) => {
                    self.record_connect_failed();
                    error!("Connection failed: {}", e);

                    // Check if we should retry
                    if !self.can_retry() {
                        warn!("Max reconnection attempts reached");
                        return Err(format!("Max reconnection attempts reached: {}", e));
                    }

                    let delay = self.next_delay();
                    warn!("Retrying connection in {:?}", delay);
                    self.record_reconnect_attempt();

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Create a streaming connection manager.
    ///
    /// This method returns a `ConnectionManager` and a watch receiver
    /// that can be used to monitor the connection state.
    pub fn stream(&self) -> watch::Receiver<ConnectionState> {
        self.last_state.clone()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_creation() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[test]
    fn test_connection_state_is_active() {
        assert!(!ConnectionState::Disconnected.is_active());
        assert!(!ConnectionState::Connecting.is_active());
        assert!(ConnectionState::Connected.is_active());
        assert!(!ConnectionState::Reconnecting.is_active());
        assert!(!ConnectionState::Failed.is_active());
    }

    #[test]
    fn test_connection_state_is_connecting() {
        assert!(!ConnectionState::Disconnected.is_connecting());
        assert!(ConnectionState::Connecting.is_connecting());
        assert!(!ConnectionState::Connected.is_connecting());
        assert!(ConnectionState::Reconnecting.is_connecting());
        assert!(!ConnectionState::Failed.is_connecting());
    }

    #[test]
    fn test_connection_state_is_lost() {
        assert!(ConnectionState::Disconnected.is_lost());
        assert!(ConnectionState::Connecting.is_lost());
        assert!(!ConnectionState::Connected.is_lost());
        assert!(ConnectionState::Reconnecting.is_lost());
        assert!(ConnectionState::Failed.is_lost());
    }

    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(300));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.use_jitter);
    }

    #[test]
    fn test_connection_manager_creation() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_connection_state_transitions() {
        let manager = ConnectionManager::new();

        // Start disconnected
        assert_eq!(manager.state(), ConnectionState::Disconnected);

        // Record connection
        manager.record_connect();
        assert_eq!(manager.state(), ConnectionState::Connected);

        // Record disconnect
        manager.record_disconnect();
        assert_eq!(manager.state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let config = ConnectionConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            ..Default::default()
        };
        let manager = ConnectionManager::with_config(config);

        // First delay should be 1 second
        let delay1 = manager.next_delay();
        assert!(delay1 >= Duration::from_secs(1));

        // Second delay should be 2 seconds
        let delay2 = manager.next_delay();
        assert!(delay2 >= Duration::from_secs(2));

        // Third delay should be 4 seconds
        let delay3 = manager.next_delay();
        assert!(delay3 >= Duration::from_secs(4));
    }

    #[tokio::test]
    async fn test_max_delay_cap() {
        let config = ConnectionConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 10.0, // Will exceed max quickly
            ..Default::default()
        };
        let manager = ConnectionManager::with_config(config);

        // First delay: 1
        manager.next_delay();

        // Second delay: 10 (capped at max)
        let delay = manager.next_delay();
        assert!(delay <= Duration::from_secs(10));

        // Third delay: still 10 (capped)
        let delay = manager.next_delay();
        assert!(delay <= Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_reset_backoff() {
        let manager = ConnectionManager::new();

        // Make several failed attempts
        manager.next_delay();
        manager.next_delay();
        manager.next_delay();

        // Reset should return to initial delay
        manager.reset_backoff();

        let delay = manager.next_delay();
        assert!(delay <= Duration::from_secs(2)); // Initial + some jitter
    }

    #[tokio::test]
    async fn test_can_retry_with_limit() {
        let config = ConnectionConfig {
            max_reconnection_attempts: 3,
            ..Default::default()
        };
        let manager = ConnectionManager::with_config(config);

        assert!(manager.can_retry());
        *manager.failed_attempts.lock().unwrap() = 1;
        assert!(manager.can_retry());
        *manager.failed_attempts.lock().unwrap() = 2;
        assert!(manager.can_retry());
        *manager.failed_attempts.lock().unwrap() = 3;
        assert!(!manager.can_retry());
    }

    #[tokio::test]
    async fn test_can_retry_unlimited() {
        let config = ConnectionConfig {
            max_reconnection_attempts: 0, // Unlimited
            ..Default::default()
        };
        let manager = ConnectionManager::with_config(config);

        assert!(manager.can_retry());
        *manager.failed_attempts.lock().unwrap() = 100;
        assert!(manager.can_retry());
    }

    #[tokio::test]
    async fn test_maintain_connection_success() {
        let manager = ConnectionManager::new();

        // Connection closure that succeeds immediately
        // Note: We use an Arc-wrapped variable to make it Fn + 'static
        let count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let count_clone = count.clone();

        let connect = move || {
            let c = count_clone.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<(), String>(())
            }
        };

        let result = manager.maintain_connection(connect).await;
        assert!(result.is_ok());
        assert_eq!(manager.state(), ConnectionState::Connected);
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_maintain_connection_max_attempts() {
        let config = ConnectionConfig {
            max_reconnection_attempts: 2,
            ..Default::default()
        };
        let manager = ConnectionManager::with_config(config);
        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        // Connection closure that always fails
        let connect = move || {
            let count = attempt_count_clone.clone();
            async move {
                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err("Connection failed".to_string())
            }
        };

        let result = manager.maintain_connection(connect).await;
        assert!(result.is_err());
        // Should have made 2 attempts (initial + 1 retry)
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert_eq!(manager.state(), ConnectionState::Failed);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let manager = ConnectionManager::new();

        manager.record_connect();
        let stats = manager.stats();
        assert!(stats.last_connected.is_some());
        assert!(stats.last_connection_start.is_some());

        manager.record_disconnect();
        let stats = manager.stats();
        assert!(stats.last_disconnected.is_some());
    }

    #[tokio::test]
    async fn test_watch_channel() {
        let manager = ConnectionManager::new();
        let mut watcher = manager.watch();

        // Initial state
        assert_eq!(*watcher.borrow(), ConnectionState::Disconnected);

        // Update state
        manager.record_connect();

        // Should see new state
        assert_eq!(*watcher.borrow(), ConnectionState::Connected);
    }
}
