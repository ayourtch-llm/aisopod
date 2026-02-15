# Issue 040: Implement Auth Profile Management

## Summary
Build an `AuthProfileManager` that manages multiple API keys per provider, supports round-robin key selection with cooldown tracking, and automatically rotates keys on rate-limit or authentication errors.

## Location
- Crate: `aisopod-provider`
- File: `crates/aisopod-provider/src/auth.rs`

## Current Behavior
There is no mechanism for managing multiple API keys for a single provider. If a key hits a rate limit or becomes invalid, requests fail without any fallback.

## Expected Behavior
After this issue is completed:
- An `AuthProfileManager` stores multiple `AuthProfile` entries per provider.
- Key selection uses round-robin rotation, skipping profiles that are in a cooldown period.
- Callers can mark a profile as good or failed; failed profiles enter a configurable cooldown before being retried.
- On rate-limit (HTTP 429) or authentication errors (HTTP 401/403), the manager automatically advances to the next available key.

## Impact
Auth profile rotation improves reliability and throughput by distributing requests across multiple API keys and gracefully handling rate limits without user intervention.

## Suggested Implementation
1. Create `crates/aisopod-provider/src/auth.rs`.
2. Define supporting types:
   ```rust
   pub struct AuthProfile {
       pub id: String,
       pub provider_id: String,
       pub api_key: String,
       pub status: ProfileStatus,
       pub last_used: Option<Instant>,
       pub cooldown_until: Option<Instant>,
   }

   pub enum ProfileStatus {
       Good,
       RateLimited,
       AuthFailed,
       Unknown,
   }
   ```
3. Define the `AuthProfileManager` struct:
   ```rust
   pub struct AuthProfileManager {
       profiles: HashMap<String, Vec<AuthProfile>>,   // provider_id -> profiles
       current_index: HashMap<String, usize>,          // round-robin index per provider
       default_cooldown: Duration,
   }
   ```
4. Implement methods:
   - `new(default_cooldown: Duration) -> Self` — create with a default cooldown duration.
   - `add_profile(&mut self, profile: AuthProfile)` — register a key for a provider.
   - `next_key(&mut self, provider_id: &str) -> Option<&AuthProfile>` — return the next available key using round-robin, skipping profiles whose `cooldown_until` is in the future.
   - `mark_good(&mut self, provider_id: &str, profile_id: &str)` — mark a profile as `Good` and update `last_used`.
   - `mark_failed(&mut self, provider_id: &str, profile_id: &str, status: ProfileStatus)` — set the profile status and start the cooldown timer.
   - `available_count(&self, provider_id: &str) -> usize` — return the number of profiles not currently in cooldown.
5. Write unit tests in the same file or in `tests/`:
   - Test that `next_key` rotates through profiles.
   - Test that a failed profile is skipped until its cooldown expires.
   - Test that `mark_good` resets a profile's status.
6. Re-export `AuthProfileManager`, `AuthProfile`, and `ProfileStatus` from `lib.rs`.
7. Run `cargo check -p aisopod-provider` and `cargo test -p aisopod-provider`.

## Dependencies
- Issue 038 (ModelProvider trait and core types)
- Issue 016 (Core configuration types — cooldown and auth settings)

## Acceptance Criteria
- [ ] `AuthProfileManager` stores multiple profiles per provider.
- [ ] `next_key()` implements round-robin rotation, skipping cooled-down profiles.
- [ ] `mark_failed()` places a profile into cooldown for the configured duration.
- [ ] `mark_good()` resets a profile to `Good` status.
- [ ] `available_count()` correctly reflects the number of usable profiles.
- [ ] Unit tests cover rotation, cooldown, and recovery scenarios.
- [ ] `cargo check -p aisopod-provider` passes.

---
*Created: 2026-02-15*
