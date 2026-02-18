//! Auth rotation tests for AuthProfileManager.
//!
//! These tests verify that:
//! - Round-robin rotation works correctly across multiple profiles
//! - Failed profiles are skipped during cooldown
//! - Cooldown expiration and profile recovery works
//! - Behavior when all profiles are in cooldown (returns None)

use std::time::Duration;

use aisopod_provider::auth::{AuthProfile, AuthProfileManager, ProfileStatus};

// ============================================================================
// Round-Robin Rotation Tests
// ============================================================================

#[test]
fn test_round_robin_two_profiles() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "profile_1".to_string(),
        "openai".to_string(),
        "sk-test-1".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "profile_2".to_string(),
        "openai".to_string(),
        "sk-test-2".to_string(),
    ));

    // First call should return profile 1
    let key1 = manager.next_key("openai");
    assert!(key1.is_some());
    assert_eq!(key1.unwrap().api_key, "sk-test-1");

    // Second call should return profile 2 (round-robin)
    let key2 = manager.next_key("openai");
    assert!(key2.is_some());
    assert_eq!(key2.unwrap().api_key, "sk-test-2");

    // Third call should cycle back to profile 1
    let key3 = manager.next_key("openai");
    assert!(key3.is_some());
    assert_eq!(key3.unwrap().api_key, "sk-test-1");
}

#[test]
fn test_round_robin_three_profiles() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "p1".to_string(),
        "openai".to_string(),
        "sk-1".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "p2".to_string(),
        "openai".to_string(),
        "sk-2".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "p3".to_string(),
        "openai".to_string(),
        "sk-3".to_string(),
    ));

    let mut keys = Vec::new();
    for _ in 0..6 {
        let key = manager.next_key("openai").unwrap();
        keys.push(key.api_key.clone());
    }

    // Should cycle through p1, p2, p3, p1, p2, p3
    assert_eq!(keys[0], "sk-1");
    assert_eq!(keys[1], "sk-2");
    assert_eq!(keys[2], "sk-3");
    assert_eq!(keys[3], "sk-1");
    assert_eq!(keys[4], "sk-2");
    assert_eq!(keys[5], "sk-3");
}

#[test]
fn test_round_robin_single_profile() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "single".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    // Should always return the same profile
    for _ in 0..5 {
        let key = manager.next_key("openai").unwrap();
        assert_eq!(key.api_key, "sk-test");
    }
}

// ============================================================================
// Cooldown and Failure Tests
// ============================================================================

#[test]
fn test_skips_failed_profile_during_cooldown() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "good".to_string(),
        "openai".to_string(),
        "sk-good".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "bad".to_string(),
        "openai".to_string(),
        "sk-bad".to_string(),
    ));

    // Mark the second profile as failed
    manager.mark_failed("openai", "bad", ProfileStatus::RateLimited);

    // Should always return the good profile
    for _ in 0..5 {
        let key = manager.next_key("openai").unwrap();
        assert_eq!(key.api_key, "sk-good");
    }
}

#[test]
fn test_all_profiles_failed_returns_none() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "p1".to_string(),
        "openai".to_string(),
        "sk-1".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "p2".to_string(),
        "openai".to_string(),
        "sk-2".to_string(),
    ));

    // Mark both as failed
    manager.mark_failed("openai", "p1", ProfileStatus::RateLimited);
    manager.mark_failed("openai", "p2", ProfileStatus::AuthFailed);

    // No available profiles
    assert!(manager.next_key("openai").is_none());
}

#[test]
fn test_rate_limited_profile_not_available() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    manager.mark_failed("openai", "profile", ProfileStatus::RateLimited);

    assert!(manager.next_key("openai").is_none());
}

#[test]
fn test_auth_failed_profile_not_available() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    manager.mark_failed("openai", "profile", ProfileStatus::AuthFailed);

    assert!(manager.next_key("openai").is_none());
}

// ============================================================================
// Cooldown Expiration Tests
// ============================================================================

#[test]
fn test_profile_recovers_after_cooldown() {
    // Use very short cooldown for testing
    let mut manager = AuthProfileManager::new(Duration::from_millis(20));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    // Mark as failed
    manager.mark_failed("openai", "profile", ProfileStatus::RateLimited);

    // Should not be available immediately
    assert!(manager.next_key("openai").is_none());

    // Wait for cooldown to expire
    std::thread::sleep(Duration::from_millis(30));

    // Should be recovered now
    let key = manager.next_key("openai");
    assert!(key.is_some());
    assert_eq!(key.unwrap().api_key, "sk-test");
}

#[test]
fn test_mark_good_resets_profile() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    // Mark as failed first
    manager.mark_failed("openai", "profile", ProfileStatus::RateLimited);
    assert!(manager.next_key("openai").is_none());

    // Mark as good
    manager.mark_good("openai", "profile");

    // Should be available now
    let key = manager.next_key("openai");
    assert!(key.is_some());
    assert_eq!(key.unwrap().api_key, "sk-test");
}

#[test]
fn test_cooldown_reset_on_success() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    // Mark as failed
    manager.mark_failed("openai", "profile", ProfileStatus::RateLimited);
    assert!(manager.next_key("openai").is_none());

    // Mark as good
    manager.mark_good("openai", "profile");

    // Now mark as failed again
    manager.mark_failed("openai", "profile", ProfileStatus::AuthFailed);

    // Should not be available again
    assert!(manager.next_key("openai").is_none());
}

// ============================================================================
// Profile Count Tests
// ============================================================================

#[test]
fn test_total_count() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "p1".to_string(),
        "openai".to_string(),
        "sk-1".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "p2".to_string(),
        "openai".to_string(),
        "sk-2".to_string(),
    ));

    assert_eq!(manager.total_count("openai"), 2);
}

#[test]
fn test_available_count() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "good1".to_string(),
        "openai".to_string(),
        "sk-1".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "good2".to_string(),
        "openai".to_string(),
        "sk-2".to_string(),
    ));

    assert_eq!(manager.available_count("openai"), 2);
}

#[test]
fn test_available_count_with_failed() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "good".to_string(),
        "openai".to_string(),
        "sk-good".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "bad".to_string(),
        "openai".to_string(),
        "sk-bad".to_string(),
    ));

    manager.mark_failed("openai", "bad", ProfileStatus::RateLimited);

    assert_eq!(manager.available_count("openai"), 1);
}

// ============================================================================
// Profile ID Tests
// ============================================================================

#[test]
fn test_profile_ids() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "p1".to_string(),
        "openai".to_string(),
        "sk-1".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "p2".to_string(),
        "openai".to_string(),
        "sk-2".to_string(),
    ));

    let ids = manager.profile_ids("openai");
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"p1"));
    assert!(ids.contains(&"p2"));
}

#[test]
fn test_profile_ids_empty_provider() {
    let manager = AuthProfileManager::new(Duration::from_secs(60));

    let ids = manager.profile_ids("nonexistent");
    assert!(ids.is_empty());
}

// ============================================================================
// Is Cooldown Complete Tests
// ============================================================================

#[test]
fn test_is_cooldown_complete_false() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    manager.mark_failed("openai", "profile", ProfileStatus::RateLimited);

    assert!(!manager.is_cooldown_complete("openai", "profile"));
}

#[test]
fn test_is_cooldown_complete_true() {
    let mut manager = AuthProfileManager::new(Duration::from_millis(10));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    manager.mark_failed("openai", "profile", ProfileStatus::RateLimited);

    // Wait for cooldown to expire
    std::thread::sleep(Duration::from_millis(20));

    assert!(manager.is_cooldown_complete("openai", "profile"));
}

// ============================================================================
// Recover Profile Tests
// ============================================================================

#[test]
fn test_recover_profile_returns_true_on_recovery() {
    let mut manager = AuthProfileManager::new(Duration::from_millis(10));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    manager.mark_failed("openai", "profile", ProfileStatus::RateLimited);

    // Should not be recoverable yet
    assert!(!manager.recover_profile("openai", "profile"));

    // Wait for cooldown
    std::thread::sleep(Duration::from_millis(20));

    // Should recover
    assert!(manager.recover_profile("openai", "profile"));

    // Should be available now
    let key = manager.next_key("openai");
    assert!(key.is_some());
}

#[test]
fn test_recover_profile_false_if_not_needed() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    // Should return false since profile is already good
    assert!(!manager.recover_profile("openai", "profile"));
}

// ============================================================================
// Provider Isolation Tests
// ============================================================================

#[test]
fn test_profiles_are_provider_isolated() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "openai-p1".to_string(),
        "openai".to_string(),
        "sk-openai-1".to_string(),
    ));
    manager.add_profile(AuthProfile::new(
        "anthropic-p1".to_string(),
        "anthropic".to_string(),
        "sk-anthropic-1".to_string(),
    ));

    // Get keys for each provider
    let openai_key = manager.next_key("openai").unwrap();
    let anthropic_key = manager.next_key("anthropic").unwrap();

    assert_eq!(openai_key.api_key, "sk-openai-1");
    assert_eq!(anthropic_key.api_key, "sk-anthropic-1");

    // Mark openai as failed
    manager.mark_failed("openai", "openai-p1", ProfileStatus::RateLimited);

    // Openai should have no keys, anthropic should still work
    assert!(manager.next_key("openai").is_none());
    assert!(manager.next_key("anthropic").is_some());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_provider_list() {
    let manager = AuthProfileManager::new(Duration::from_secs(60));

    assert!(manager.next_key("nonexistent").is_none());
}

#[test]
fn test_zero_duration_cooldown() {
    let mut manager = AuthProfileManager::new(Duration::ZERO);

    manager.add_profile(AuthProfile::new(
        "profile".to_string(),
        "openai".to_string(),
        "sk-test".to_string(),
    ));

    manager.mark_failed("openai", "profile", ProfileStatus::RateLimited);

    // With zero cooldown, the profile should be immediately unavailable
    // (the cooldown_until will be set to now, which is >= now)
    // Actually, with zero duration, it should still be unavailable because
    // we check >= now, and now == now is true
    assert!(manager.next_key("openai").is_none());
}

#[test]
fn test_profile_with_no_api_key() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    manager.add_profile(AuthProfile::new(
        "empty".to_string(),
        "openai".to_string(),
        "".to_string(),
    ));

    // Should still be able to get the profile (empty key is valid)
    let key = manager.next_key("openai");
    assert!(key.is_some());
    assert!(key.unwrap().api_key.is_empty());
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_high_frequency_rotation() {
    let mut manager = AuthProfileManager::new(Duration::from_secs(60));

    // Add 10 profiles
    for i in 0..10 {
        manager.add_profile(AuthProfile::new(
            format!("p{}", i),
            "openai".to_string(),
            format!("sk-{}", i),
        ));
    }

    // Make 100 requests and verify we cycle through all profiles
    let mut counts = std::collections::HashMap::new();
    for _ in 0..100 {
        let key = manager.next_key("openai").unwrap();
        *counts.entry(&key.api_key).or_insert(0) += 1;
    }

    // Each profile should be used approximately 10 times
    for (key, count) in counts {
        assert!(count >= 5, "Key {} was used {} times (expected ~10)", key, count);
    }
}
