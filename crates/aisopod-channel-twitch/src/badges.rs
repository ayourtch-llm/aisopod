//! Twitch badge parsing and status detection.
//!
//! This module provides functionality for parsing Twitch badges from
//! message tags and detecting user status (moderator, subscriber, etc.)

use serde::{Deserialize, Serialize};

/// Represents a Twitch badge that a user has.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Badge {
    /// The name of the badge (e.g., "moderator", "subscriber", "broadcaster")
    pub name: String,
    /// The version of the badge
    pub version: String,
}

impl Badge {
    /// Create a new badge with the given name and version.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Parse badges from a comma-separated string.
///
/// Twitch sends badges in the format "moderator/1,subscriber/12"
/// where the first part is the badge name and the second is the version.
///
/// # Arguments
///
/// * `badge_str` - The comma-separated badge string
///
/// # Returns
///
/// A vector of parsed badges.
///
/// # Examples
///
/// ```
/// use aisopod_channel_twitch::parse_badges;
///
/// let badges = parse_badges("moderator/1,subscriber/12");
/// assert_eq!(badges.len(), 2);
/// assert_eq!(badges[0].name, "moderator");
/// assert_eq!(badges[1].name, "subscriber");
/// ```
pub fn parse_badges(badge_str: &str) -> Vec<Badge> {
    if badge_str.is_empty() {
        return Vec::new();
    }

    badge_str
        .split(',')
        .filter_map(|b| {
            let mut parts = b.split('/');
            let name = parts.next()?;
            let version = parts.next()?;
            Some(Badge::new(name, version))
        })
        .collect()
}

/// Check if a user is a moderator based on their badges.
///
/// This includes both regular moderators and the broadcaster (stream owner).
///
/// # Arguments
///
/// * `badges` - The list of badges to check
///
/// # Returns
///
/// `true` if the user is a moderator or broadcaster, `false` otherwise.
pub fn is_moderator(badges: &[Badge]) -> bool {
    badges
        .iter()
        .any(|b| b.name == "moderator" || b.name == "broadcaster")
}

/// Check if a user is a subscriber based on their badges.
///
/// # Arguments
///
/// * `badges` - The list of badges to check
///
/// # Returns
///
/// `true` if the user has a subscriber badge, `false` otherwise.
pub fn is_subscriber(badges: &[Badge]) -> bool {
    badges.iter().any(|b| b.name == "subscriber")
}

/// Check if a user is the broadcaster (stream owner) based on their badges.
///
/// # Arguments
///
/// * `badges` - The list of badges to check
///
/// # Returns
///
/// `true` if the user is the broadcaster, `false` otherwise.
pub fn is_broadcaster(badges: &[Badge]) -> bool {
    badges.iter().any(|b| b.name == "broadcaster")
}

/// Check if a user is a VIP based on their badges.
///
/// # Arguments
///
/// * `badges` - The list of badges to check
///
/// # Returns
///
/// `true` if the user is a VIP, `false` otherwise.
pub fn is_vip(badges: &[Badge]) -> bool {
    badges.iter().any(|b| b.name == "vip")
}

/// Parse badge attributes from a semicolon-separated attributes string.
///
/// Some badges come with attributes like "bits/100" for bits badges.
///
/// # Arguments
///
/// * `badge_info` - The semicolon-separated badge info string
///
/// # Returns
///
/// A map of badge names to their info strings.
///
/// # Examples
///
/// ```
/// use aisopod_channel_twitch::parse_badge_info;
///
/// let info = parse_badge_info("moderator=1;subscriber=12");
/// assert_eq!(info.get("moderator"), Some(&"1".to_string()));
/// assert_eq!(info.get("subscriber"), Some(&"12".to_string()));
/// ```
pub fn parse_badge_info(badge_info: &str) -> std::collections::HashMap<String, String> {
    if badge_info.is_empty() {
        return std::collections::HashMap::new();
    }

    badge_info
        .split(';')
        .filter_map(|b| {
            let mut parts = b.split('=');
            let name = parts.next()?;
            let value = parts.next()?;
            Some((name.to_string(), value.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_badges_basic() {
        let badges = parse_badges("moderator/1,subscriber/12");
        assert_eq!(badges.len(), 2);
        assert_eq!(badges[0].name, "moderator");
        assert_eq!(badges[0].version, "1");
        assert_eq!(badges[1].name, "subscriber");
        assert_eq!(badges[1].version, "12");
    }

    #[test]
    fn test_parse_badges_empty() {
        let badges = parse_badges("");
        assert!(badges.is_empty());
    }

    #[test]
    fn test_parse_badges_no_version() {
        let badges = parse_badges("moderator/");
        assert_eq!(badges.len(), 1);
        assert_eq!(badges[0].name, "moderator");
        assert_eq!(badges[0].version, "");
    }

    #[test]
    fn test_is_moderator() {
        let badges = parse_badges("moderator/1,subscriber/12");
        assert!(is_moderator(&badges));
    }

    #[test]
    fn test_is_moderator_broadcaster() {
        let badges = parse_badges("broadcaster/1");
        assert!(is_moderator(&badges));
    }

    #[test]
    fn test_is_moderator_no_moderator() {
        let badges = parse_badges("subscriber/12,premium/1");
        assert!(!is_moderator(&badges));
    }

    #[test]
    fn test_is_subscriber() {
        let badges = parse_badges("moderator/1,subscriber/12");
        assert!(is_subscriber(&badges));
    }

    #[test]
    fn test_is_subscriber_no_subscriber() {
        let badges = parse_badges("moderator/1,premium/1");
        assert!(!is_subscriber(&badges));
    }

    #[test]
    fn test_is_broadcaster() {
        let badges = parse_badges("broadcaster/1");
        assert!(is_broadcaster(&badges));
    }

    #[test]
    fn test_is_broadcaster_not_broadcaster() {
        let badges = parse_badges("moderator/1");
        assert!(!is_broadcaster(&badges));
    }

    #[test]
    fn test_is_vip() {
        let badges = parse_badges("vip/1");
        assert!(is_vip(&badges));
    }

    #[test]
    fn test_parse_badge_info() {
        let info = parse_badge_info("moderator=1;subscriber=12");
        assert_eq!(info.get("moderator"), Some(&"1".to_string()));
        assert_eq!(info.get("subscriber"), Some(&"12".to_string()));
    }

    #[test]
    fn test_parse_badge_info_empty() {
        let info = parse_badge_info("");
        assert!(info.is_empty());
    }

    #[test]
    fn test_badge_serialization() {
        let badge = Badge::new("moderator", "1");
        let json = serde_json::to_string(&badge).unwrap();
        assert!(json.contains("moderator"));
        assert!(json.contains("1"));
    }
}
