//! Mock signal-cli for integration testing.
//!
//! This module provides a mock implementation of signal-cli that responds
//! with predefined JSON messages, allowing integration tests to verify
//! Signal channel behavior without requiring a real signal-cli installation.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// A mock signal-cli that responds with predefined JSON messages.
///
/// This struct creates a temporary executable script that mimics
/// signal-cli's JSON-RPC interface for testing purposes.
pub struct MockSignalCli {
    /// Temporary directory containing the mock script
    _temp_dir: TempDir,
    /// Path to the mock script
    script_path: PathBuf,
}

impl MockSignalCli {
    /// Creates a new mock signal-cli instance.
    ///
    /// This creates a temporary directory with an executable script
    /// that responds with mock JSON-RPC responses.
    ///
    /// # Returns
    ///
    /// A MockSignalCli instance with the path to the mock script.
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("signal-cli");
        
        // Create a mock signal-cli script that returns predefined responses
        let script_content = format!(
            "#!/bin/bash\n\
             # Mock signal-cli for testing\n\
             \n\
             # Read stdin for JSON-RPC requests\n\
             while IFS= read -r line; do\n\
                 # Parse the request type from the line\n\
                 if echo \"$line\" | grep -q '\"type\"' 2>/dev/null; then\n\
                     # This is a JSON-RPC request\n\
                     request_type=$(echo \"$line\" | grep -o '\"type\"[[:space:]]*:[[:space:]]*\"[^\"]*\"' | sed 's/.*: *\"\\([^\"]*\\)\".*/\\1/')\n\
                     \n\
                     case \"$request_type\" in\n\
                         \"receive\")\n\
                             # Simulate receiving a message\n\
                             echo '{{\"type\":\"receive\",\"source\":\"+1234567890\",\"timestamp\":1618907555000,\"message\":{{\"body\":\"Hello from Signal\"}},\"id\":\"msg123\"}}'\n\
                             ;;\n\
                         \"sent\")\n\
                             # Simulate sent message confirmation\n\
                             echo '{{\"type\":\"sent\",\"timestamp\":1618907555000,\"message\":{{\"body\":\"Test message\"}},\"id\":\"msg456\"}}'\n\
                             ;;\n\
                         \"groupReceive\")\n\
                             # Simulate receiving a group message\n\
                             echo '{{\"type\":\"groupReceive\",\"groupId\":\"test-group\",\"source\":\"+1234567890\",\"timestamp\":1618907555000,\"message\":{{\"body\":\"Group message\"}},\"id\":\"msg789\"}}'\n\
                             ;;\n\
                         *)\n\
                             # Default response for unknown requests\n\
                             echo '{{\"error\":\"Unknown request type\"}}'\n\
                             ;;\n\
                     esac\n\
                 else\n\
                     # Not a JSON-RPC request, just echo back what we got\n\
                     echo \"$line\"\n\
                 fi\n\
             done\n",
        );
        
        fs::write(&script_path, &script_content)
            .expect("Failed to write mock script");
        
        // Make the script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
                .expect("Failed to make script executable");
        }
        #[cfg(windows)]
        {
            // On Windows, just ensure the file exists (executability is handled differently)
        }
        
        Self {
            _temp_dir: temp_dir,
            script_path,
        }
    }
    
    /// Returns the path to the mock signal-cli script.
    ///
    /// # Returns
    ///
    /// The path as a string slice.
    pub fn path(&self) -> &str {
        self.script_path.to_str().unwrap_or("")
    }
}

impl Default for MockSignalCli {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_signal_cli_exists() {
        let mock = MockSignalCli::new();
        assert!(mock.path().len() > 0);
    }
    
    #[test]
    fn test_mock_signal_cli_path() {
        let mock = MockSignalCli::new();
        let path = mock.path();
        assert!(path.contains("signal-cli"));
    }
}
