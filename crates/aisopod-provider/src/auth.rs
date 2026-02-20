//! Auth profile management for handling multiple API keys per provider.
//!
//! This module provides an `AuthProfileManager` that manages multiple API keys
//! per provider, supports round-robin key selection with cooldown tracking,
//! and automatically rotates keys on rate-limit or authentication errors.

use std::collections::HashMap;
use std::time::Instant;

/// Status of an authentication profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileStatus {
    /// Profile is in good standing.
    Good,
    /// Profile has hit a rate limit.
    RateLimited,
    /// Profile has authentication failures.
    AuthFailed,
    /// Profile status is unknown.
    Unknown,
}

/// An authentication profile for a provider API key.
#[derive(Debug, Clone)]
pub struct AuthProfile {
    /// Unique identifier for this profile.
    pub id: String,
    /// The provider ID this profile belongs to.
    pub provider_id: String,
    /// The API key for this profile.
    pub api_key: String,
    /// Current status of this profile.
    pub status: ProfileStatus,
    /// When this profile was last used.
    pub last_used: Option<Instant>,
    /// When this profile's cooldown period ends (if any).
    pub cooldown_until: Option<Instant>,
}

impl AuthProfile {
    /// Creates a new authentication profile.
    pub fn new(id: String, provider_id: String, api_key: String) -> Self {
        Self {
            id,
            provider_id,
            api_key,
            status: ProfileStatus::Good,
            last_used: None,
            cooldown_until: None,
        }
    }

    /// Checks if this profile is currently available (not in cooldown).
    pub fn is_available(&self) -> bool {
        self.status == ProfileStatus::Good
            && self.cooldown_until.map_or(true, |t| Instant::now() >= t)
    }
}

/// Manager for authentication profiles with round-robin rotation and cooldown support.
#[derive(Debug)]
pub struct AuthProfileManager {
    /// Profiles stored by provider ID.
    profiles: HashMap<String, Vec<AuthProfile>>,
    /// Round-robin index for each provider.
    current_index: HashMap<String, usize>,
    /// Default cooldown duration for failed profiles.
    default_cooldown: std::time::Duration,
}

impl AuthProfileManager {
    /// Creates a new `AuthProfileManager` with the specified default cooldown duration.
    pub fn new(default_cooldown: std::time::Duration) -> Self {
        Self {
            profiles: HashMap::new(),
            current_index: HashMap::new(),
            default_cooldown,
        }
    }

    /// Adds an authentication profile for a provider.
    pub fn add_profile(&mut self, profile: AuthProfile) {
        let provider_id = profile.provider_id.clone();
        self.profiles
            .entry(provider_id.clone())
            .or_insert_with(Vec::new)
            .push(profile);
        // Initialize or reset the round-robin index for this provider
        self.current_index.entry(provider_id).or_insert(0);
    }

    /// Returns the next available key for a provider using round-robin rotation,
    /// skipping profiles that are in a cooldown period or failed.
    pub fn next_key(&mut self, provider_id: &str) -> Option<&AuthProfile> {
        let profiles = self.profiles.get_mut(provider_id)?;
        let start_index = *self
            .current_index
            .entry(provider_id.to_string())
            .or_insert(0);
        let len = profiles.len();

        if len == 0 {
            return None;
        }

        // First, try to recover any profiles that have completed their cooldown
        for profile in profiles.iter_mut() {
            if profile
                .cooldown_until
                .map_or(false, |t| Instant::now() >= t)
            {
                profile.status = ProfileStatus::Good;
                profile.cooldown_until = None;
            }
        }

        // Search through all profiles starting from current index
        for i in 0..len {
            let idx = (start_index + i) % len;
            if profiles[idx].is_available() {
                *self
                    .current_index
                    .entry(provider_id.to_string())
                    .or_insert(0) = (idx + 1) % len;
                return Some(&profiles[idx]);
            }
        }

        // No available profiles found, reset index to 0 for next time
        *self
            .current_index
            .entry(provider_id.to_string())
            .or_insert(0) = 0;
        None
    }

    /// Marks a profile as successfully used.
    pub fn mark_good(&mut self, provider_id: &str, profile_id: &str) {
        if let Some(profiles) = self.profiles.get_mut(provider_id) {
            if let Some(profile) = profiles.iter_mut().find(|p| p.id == profile_id) {
                profile.status = ProfileStatus::Good;
                profile.last_used = Some(Instant::now());
                profile.cooldown_until = None;
            }
        }
    }

    /// Marks a profile as failed and starts a cooldown period.
    pub fn mark_failed(&mut self, provider_id: &str, profile_id: &str, status: ProfileStatus) {
        if let Some(profiles) = self.profiles.get_mut(provider_id) {
            if let Some(profile) = profiles.iter_mut().find(|p| p.id == profile_id) {
                profile.status = status;
                profile.cooldown_until = Some(Instant::now() + self.default_cooldown);
            }
        }
    }

    /// Checks if a profile has completed its cooldown period.
    pub fn is_cooldown_complete(&self, provider_id: &str, profile_id: &str) -> bool {
        self.profiles
            .get(provider_id)
            .and_then(|profiles| profiles.iter().find(|p| p.id == profile_id))
            .map(|profile| profile.cooldown_until.map_or(true, |t| Instant::now() >= t))
            .unwrap_or(false)
    }

    /// Recovers a failed profile by resetting its status to Good after cooldown expires.
    pub fn recover_profile(&mut self, provider_id: &str, profile_id: &str) -> bool {
        if let Some(profiles) = self.profiles.get_mut(provider_id) {
            if let Some(profile) = profiles.iter_mut().find(|p| p.id == profile_id) {
                if profile.cooldown_until.map_or(true, |t| Instant::now() >= t) {
                    profile.status = ProfileStatus::Good;
                    profile.cooldown_until = None;
                    return true;
                }
            }
        }
        false
    }

    /// Returns the number of profiles for a provider that are not currently in cooldown.
    pub fn available_count(&self, provider_id: &str) -> usize {
        self.profiles
            .get(provider_id)
            .map(|profiles| profiles.iter().filter(|p| p.is_available()).count())
            .unwrap_or(0)
    }

    /// Returns the total number of profiles for a provider (including those in cooldown).
    pub fn total_count(&self, provider_id: &str) -> usize {
        self.profiles
            .get(provider_id)
            .map(|profiles| profiles.len())
            .unwrap_or(0)
    }

    /// Returns a list of all profile IDs for a provider.
    pub fn profile_ids(&self, provider_id: &str) -> Vec<&str> {
        self.profiles
            .get(provider_id)
            .map(|profiles| profiles.iter().map(|p| p.id.as_str()).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_auth_profile_new() {
        let profile = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-test-key".to_string(),
        );

        assert_eq!(profile.id, "profile_1");
        assert_eq!(profile.provider_id, "openai");
        assert_eq!(profile.api_key, "sk-test-key");
        assert_eq!(profile.status, ProfileStatus::Good);
        assert!(profile.last_used.is_none());
        assert!(profile.cooldown_until.is_none());
    }

    #[test]
    fn test_auth_profile_is_available() {
        let now = Instant::now();

        // Good profile without cooldown
        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-test-key".to_string(),
        );
        assert!(profile1.is_available());

        // Good profile with expired cooldown
        let profile2 = AuthProfile {
            id: "profile_2".to_string(),
            provider_id: "openai".to_string(),
            api_key: "sk-test-key".to_string(),
            status: ProfileStatus::Good,
            last_used: Some(now),
            cooldown_until: Some(now - Duration::from_secs(1)),
        };
        assert!(profile2.is_available());

        // Good profile with active cooldown
        let profile3 = AuthProfile {
            id: "profile_3".to_string(),
            provider_id: "openai".to_string(),
            api_key: "sk-test-key".to_string(),
            status: ProfileStatus::Good,
            last_used: Some(now),
            cooldown_until: Some(now + Duration::from_secs(1)),
        };
        assert!(!profile3.is_available());

        // RateLimited profile
        let profile4 = AuthProfile {
            id: "profile_4".to_string(),
            provider_id: "openai".to_string(),
            api_key: "sk-test-key".to_string(),
            status: ProfileStatus::RateLimited,
            last_used: Some(now),
            cooldown_until: None,
        };
        assert!(!profile4.is_available());

        // AuthFailed profile
        let profile5 = AuthProfile {
            id: "profile_5".to_string(),
            provider_id: "openai".to_string(),
            api_key: "sk-test-key".to_string(),
            status: ProfileStatus::AuthFailed,
            last_used: Some(now),
            cooldown_until: None,
        };
        assert!(!profile5.is_available());
    }

    #[test]
    fn test_auth_profile_manager_new() {
        let manager = AuthProfileManager::new(Duration::from_secs(60));
        assert_eq!(manager.default_cooldown, Duration::from_secs(60));
    }

    #[test]
    fn test_add_profile() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        manager.add_profile(profile1);

        assert_eq!(manager.total_count("openai"), 1);
        assert_eq!(manager.available_count("openai"), 1);
    }

    #[test]
    fn test_add_multiple_profiles_same_provider() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );
        let profile3 = AuthProfile::new(
            "profile_3".to_string(),
            "openai".to_string(),
            "sk-key-3".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);
        manager.add_profile(profile3);

        assert_eq!(manager.total_count("openai"), 3);
        assert_eq!(manager.available_count("openai"), 3);
    }

    #[test]
    fn test_add_profiles_different_providers() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "anthropic".to_string(),
            "sk-key-2".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);

        assert_eq!(manager.total_count("openai"), 1);
        assert_eq!(manager.total_count("anthropic"), 1);
        assert_eq!(manager.available_count("openai"), 1);
        assert_eq!(manager.available_count("anthropic"), 1);
    }

    #[test]
    fn test_next_key_round_robin() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);

        // First call should return profile_1 (index 0)
        let key1 = manager.next_key("openai");
        assert!(key1.is_some());
        assert_eq!(key1.unwrap().id, "profile_1");

        // Second call should return profile_2 (index 1)
        let key2 = manager.next_key("openai");
        assert!(key2.is_some());
        assert_eq!(key2.unwrap().id, "profile_2");

        // Third call should rotate back to profile_1 (index 0)
        let key3 = manager.next_key("openai");
        assert!(key3.is_some());
        assert_eq!(key3.unwrap().id, "profile_1");
    }

    #[test]
    fn test_next_key_skips_failed() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);

        // Mark profile_1 as failed
        manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);

        // next_key should skip profile_1 and return profile_2
        let key = manager.next_key("openai");
        assert!(key.is_some());
        assert_eq!(key.unwrap().id, "profile_2");
    }

    #[test]
    fn test_next_key_skips_cooldown() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );
        let profile3 = AuthProfile::new(
            "profile_3".to_string(),
            "openai".to_string(),
            "sk-key-3".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);
        manager.add_profile(profile3);

        // Mark profile_1 as failed with cooldown
        manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);

        // next_key should skip profile_1 and return profile_2
        let key1 = manager.next_key("openai");
        assert!(key1.is_some());
        assert_eq!(key1.unwrap().id, "profile_2");

        // Mark profile_2 as failed
        manager.mark_failed("openai", "profile_2", ProfileStatus::AuthFailed);

        // next_key should skip profile_1 and profile_2, return profile_3
        let key2 = manager.next_key("openai");
        assert!(key2.is_some());
        assert_eq!(key2.unwrap().id, "profile_3");
    }

    #[test]
    fn test_next_key_all_failed() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);

        // Mark both as failed
        manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);
        manager.mark_failed("openai", "profile_2", ProfileStatus::AuthFailed);

        // next_key should return None when all profiles are unavailable
        let key = manager.next_key("openai");
        assert!(key.is_none());
    }

    #[test]
    fn test_next_key_no_profiles() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        // No profiles added
        let key = manager.next_key("openai");
        assert!(key.is_none());
    }

    #[test]
    fn test_mark_good_resets_status() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        manager.add_profile(profile1);

        // Mark as failed
        manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);
        assert_eq!(manager.total_count("openai"), 1);
        assert_eq!(manager.available_count("openai"), 0);

        // Mark as good
        manager.mark_good("openai", "profile_1");
        assert_eq!(manager.available_count("openai"), 1);

        // Should be able to get the key again
        let key = manager.next_key("openai");
        assert!(key.is_some());
        assert_eq!(key.unwrap().id, "profile_1");
    }

    #[test]
    fn test_available_count() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );
        let profile3 = AuthProfile::new(
            "profile_3".to_string(),
            "openai".to_string(),
            "sk-key-3".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);
        manager.add_profile(profile3);

        assert_eq!(manager.available_count("openai"), 3);

        // Mark one as failed
        manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);
        assert_eq!(manager.available_count("openai"), 2);

        // Mark another as failed
        manager.mark_failed("openai", "profile_2", ProfileStatus::AuthFailed);
        assert_eq!(manager.available_count("openai"), 1);

        // Mark one as good
        manager.mark_good("openai", "profile_1");
        assert_eq!(manager.available_count("openai"), 2);
    }

    #[test]
    fn test_total_count() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);

        assert_eq!(manager.total_count("openai"), 2);

        // Mark all as failed
        manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);
        manager.mark_failed("openai", "profile_2", ProfileStatus::AuthFailed);

        // Total count should still be 2
        assert_eq!(manager.total_count("openai"), 2);
        // But available count should be 0
        assert_eq!(manager.available_count("openai"), 0);
    }

    #[test]
    fn test_profile_ids() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(60));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);

        let ids = manager.profile_ids("openai");
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"profile_1"));
        assert!(ids.contains(&"profile_2"));
    }

    #[test]
    fn test_round_robin_with_recovery() {
        let mut manager = AuthProfileManager::new(Duration::from_secs(1));

        let profile1 = AuthProfile::new(
            "profile_1".to_string(),
            "openai".to_string(),
            "sk-key-1".to_string(),
        );
        let profile2 = AuthProfile::new(
            "profile_2".to_string(),
            "openai".to_string(),
            "sk-key-2".to_string(),
        );

        manager.add_profile(profile1);
        manager.add_profile(profile2);

        // Use profile_1, then mark it failed
        let key1 = manager.next_key("openai");
        assert!(key1.is_some());
        assert_eq!(key1.unwrap().id, "profile_1");
        manager.mark_failed("openai", "profile_1", ProfileStatus::RateLimited);

        // Use profile_2
        let key2 = manager.next_key("openai");
        assert!(key2.is_some());
        assert_eq!(key2.unwrap().id, "profile_2");

        // Wait for cooldown to expire
        std::thread::sleep(Duration::from_millis(1100));

        // Now profile_1 should be available again
        let key3 = manager.next_key("openai");
        assert!(key3.is_some());
        assert_eq!(key3.unwrap().id, "profile_1");
    }
}
