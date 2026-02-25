//! Security middleware for the gateway
//!
//! This module provides security hardening measures including:
//! - SecretString type for masking sensitive values in logs
//! - Input sanitization functions to prevent injection attacks
//! - Request size validation

use std::ops::Deref;

use std::fmt;

/// A wrapper type for sensitive string values that masks their output in logs
///
/// This type implements `Debug` to show `SecretString([redacted])` instead of
/// the actual value, preventing accidental exposure of secrets in log files.
#[derive(Clone)]
pub struct SecretString(String);

impl SecretString {
    /// Create a new SecretString from a regular string
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Get the inner string value (use sparingly!)
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SecretString").field(&format_args!("[redacted]")).finish()
    }
}

impl Deref for SecretString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Sanitize user input to prevent common injection attacks
///
/// This function performs basic sanitization:
/// - Trims whitespace
/// - Removes null bytes
/// - Normalizes line endings
///
/// Note: This is a basic sanitization function. For comprehensive protection,
/// use parameterized queries and input validation specific to your use case.
pub fn sanitize_input(input: &str) -> String {
    // Remove null bytes which can cause issues
    let sanitized = input.replace('\0', "");
    
    // Normalize line endings (CRLF -> LF)
    let sanitized = sanitized.replace("\r\n", "\n").replace('\r', "\n");
    
    // Trim leading/trailing whitespace
    sanitized.trim().to_string()
}

/// Validate that input does not contain common injection patterns
///
/// This function checks for:
/// - SQL injection patterns (SELECT, UNION, DROP, etc.)
/// - Command injection patterns (|, ;, &, $, backticks)
/// - XSS patterns (script tags)
///
/// Returns `Ok(())` if the input appears safe, `Err(reason)` otherwise.
pub fn validate_no_injection(input: &str) -> Result<(), String> {
    let normalized = input.to_lowercase();
    
    // SQL injection patterns
    let sql_patterns = [
        "select", "union", "insert", "update", "delete", "drop", "alter",
        "truncate", "exec", "execute", "xp_", "sp_", "1=1", "1=1--", "' or '",
        "' or 1=1", "' or ''=", "'; --", "--", "/*", "*/", "benchmark",
        "sleep", "waitfor", "having", "group by", "into", "load_file",
        "outfile", "dumpfile", "subselect", "concat",
    ];
    
    for pattern in sql_patterns {
        if normalized.contains(pattern) {
            return Err(format!("Potential SQL injection detected: '{}'", pattern));
        }
    }
    
    // XSS patterns (basic) - check BEFORE command injection to catch script tags first
    let xss_patterns = ["<script", "</script>", "javascript:", "onerror=", "onload=", "onclick="];
    
    for pattern in xss_patterns {
        if normalized.contains(pattern) {
            return Err(format!("Potential XSS injection detected: '{}'", pattern));
        }
    }
    
    // Command injection patterns
    let cmd_patterns = [
        "|", ";", "&", "$", "`", "$( ", "$(.", ">", "<", ">>", "<<",
        "| cat", "; cat", "& cat", "| ls", "; ls", "& ls",
        "| rm", "; rm", "& rm", "| wget", "; wget", "& wget",
        "| curl", "; curl", "& curl", "| nc", "; nc", "& nc",
        "| netcat", "; netcat", "& netcat", "| /bin/", "; /bin/", "& /bin/",
        "| /sh", "; /sh", "& /sh", "| sh", "; sh", "& sh",
    ];
    
    for pattern in cmd_patterns {
        if normalized.contains(pattern) {
            return Err(format!("Potential command injection detected: '{}'", pattern));
        }
    }
    
    Ok(())
}

/// Configuration for request size limits
#[derive(Debug, Clone)]
pub struct RequestSizeLimits {
    /// Maximum size of request body in bytes
    pub max_body_size: usize,
    /// Maximum size of headers in bytes
    pub max_headers_size: usize,
    /// Maximum number of headers
    pub max_headers_count: usize,
}

impl Default for RequestSizeLimits {
    fn default() -> Self {
        Self {
            max_body_size: 10 * 1024 * 1024,      // 10MB default
            max_headers_size: 8192,                // 8KB default
            max_headers_count: 100,                // 100 headers default
        }
    }
}

impl RequestSizeLimits {
    /// Create new size limits with custom values
    pub fn new(max_body_size: usize, max_headers_size: usize, max_headers_count: usize) -> Self {
        Self {
            max_body_size,
            max_headers_size,
            max_headers_count,
        }
    }
    
    /// Check if a request body size is within limits
    pub fn check_body_size(&self, size: usize) -> Result<(), String> {
        if size > self.max_body_size {
            Err(format!(
                "Request body size {} exceeds maximum of {} bytes",
                size, self.max_body_size
            ))
        } else {
            Ok(())
        }
    }
    
    /// Check if a headers size is within limits
    pub fn check_headers_size(&self, size: usize) -> Result<(), String> {
        if size > self.max_headers_size {
            Err(format!(
                "Headers size {} exceeds maximum of {} bytes",
                size, self.max_headers_size
            ))
        } else {
            Ok(())
        }
    }
    
    /// Check if headers count is within limits
    pub fn check_headers_count(&self, count: usize) -> Result<(), String> {
        if count > self.max_headers_count {
            Err(format!(
                "Headers count {} exceeds maximum of {}",
                count, self.max_headers_count
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_debug() {
        let secret = SecretString::new("my_password_123".to_string());
        let debug_str = format!("{:?}", secret);
        assert_eq!(debug_str, "SecretString([redacted])");
    }

    #[test]
    fn test_secret_string_deref() {
        let secret = SecretString::new("hello".to_string());
        assert_eq!(secret.deref(), "hello");
        assert_eq!(&*secret, "hello");
    }

    #[test]
    fn test_secret_string_partial_eq() {
        let secret1 = SecretString::new("same".to_string());
        let secret2 = SecretString::new("same".to_string());
        let secret3 = SecretString::new("different".to_string());
        
        assert!(secret1 == secret2);
        assert!(secret1 != secret3);
    }

    #[test]
    fn test_sanitize_input_basic() {
        let input = "  hello world  ";
        let sanitized = sanitize_input(input);
        assert_eq!(sanitized, "hello world");
    }

    #[test]
    fn test_sanitize_input_null_bytes() {
        let input = "hello\0world";
        let sanitized = sanitize_input(input);
        assert_eq!(sanitized, "helloworld");
    }

    #[test]
    fn test_sanitize_input_line_endings() {
        let input = "line1\r\nline2\rline3";
        let sanitized = sanitize_input(input);
        assert_eq!(sanitized, "line1\nline2\nline3");
    }

    #[test]
    fn test_sanitize_input_combined() {
        let input = "  hello\0world  \r\n";
        let sanitized = sanitize_input(input);
        assert_eq!(sanitized, "helloworld");
    }

    #[test]
    fn test_validate_no_injection_clean() {
        let clean_input = "hello world 123";
        assert!(validate_no_injection(clean_input).is_ok());
    }

    #[test]
    fn test_validate_no_injection_sql_basic() {
        let sql_input = "SELECT * FROM users";
        let result = validate_no_injection(sql_input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SQL injection"));
    }

    #[test]
    fn test_validate_no_injection_sql_case_insensitive() {
        let sql_input = "select * from users";
        let result = validate_no_injection(sql_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_injection_sql_union() {
        let sql_input = "1 UNION SELECT password FROM users";
        let result = validate_no_injection(sql_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_injection_cmd_pipe() {
        let cmd_input = "ls | cat /etc/passwd";
        let result = validate_no_injection(cmd_input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("command injection"));
    }

    #[test]
    fn test_validate_no_injection_cmd_semicolon() {
        let cmd_input = "ls; cat /etc/passwd";
        let result = validate_no_injection(cmd_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_injection_cmd_dollar() {
        let cmd_input = "$(whoami)";
        let result = validate_no_injection(cmd_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_injection_cmd_backtick() {
        let cmd_input = "`whoami`";
        let result = validate_no_injection(cmd_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_injection_xss_script() {
        let xss_input = "<script>alert('xss')</script>";
        let result = validate_no_injection(xss_input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("XSS"));
    }

    #[test]
    fn test_validate_no_injection_xss_onerror() {
        let xss_input = "<img onerror=alert(1)>";
        let result = validate_no_injection(xss_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_injection_comment_syntax() {
        let sql_input = "-- comment";
        let result = validate_no_injection(sql_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_injection_multiple_patterns() {
        let malicious_input = "'; DROP TABLE users; --";
        let result = validate_no_injection(malicious_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_request_size_limits_default() {
        let limits = RequestSizeLimits::default();
        assert_eq!(limits.max_body_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_headers_size, 8192);
        assert_eq!(limits.max_headers_count, 100);
    }

    #[test]
    fn test_request_size_limits_custom() {
        let limits = RequestSizeLimits::new(1024, 512, 50);
        assert_eq!(limits.max_body_size, 1024);
        assert_eq!(limits.max_headers_size, 512);
        assert_eq!(limits.max_headers_count, 50);
    }

    #[test]
    fn test_request_size_limits_check_body() {
        let limits = RequestSizeLimits::new(100, 50, 10);
        
        assert!(limits.check_body_size(50).is_ok());
        assert!(limits.check_body_size(100).is_ok());
        assert!(limits.check_body_size(101).is_err());
    }

    #[test]
    fn test_request_size_limits_check_headers_size() {
        let limits = RequestSizeLimits::new(1000, 100, 10);
        
        assert!(limits.check_headers_size(50).is_ok());
        assert!(limits.check_headers_size(100).is_ok());
        assert!(limits.check_headers_size(101).is_err());
    }

    #[test]
    fn test_request_size_limits_check_headers_count() {
        let limits = RequestSizeLimits::new(1000, 1000, 5);
        
        assert!(limits.check_headers_count(3).is_ok());
        assert!(limits.check_headers_count(5).is_ok());
        assert!(limits.check_headers_count(6).is_err());
    }
}
