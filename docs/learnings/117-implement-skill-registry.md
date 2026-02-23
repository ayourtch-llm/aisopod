# Issue 117: Implement SkillRegistry for Discovery and Lifecycle

## Implementation Summary

This issue implemented the `SkillRegistry` struct that serves as the central coordination point for skill discovery and lifecycle management in the aisopod system.

## Key Implementation Details

### SkillStatus Enum

The `SkillStatus` enum tracks the health and availability of skills:

```rust
pub enum SkillStatus {
    Ready,               // Skill is loaded and operational
    Degraded { reason: String },  // Loaded but missing requirements
    Failed { error: String },     // Failed to initialize
    Unloaded,             // Not loaded
}
```

**Key Design Decision**: The enum uses serializable variants (`Degraded` and `Failed` with data fields) to support status reporting via APIs and logging. This allows the system to communicate specific failure reasons to operators.

### SkillRegistry Struct

The registry maintains three HashMaps:

- `skills: HashMap<String, Arc<dyn Skill>>` - Main skill storage
- `agent_skills: HashMap<String, Vec<String>>` - Agent-to-skill mappings
- `statuses: HashMap<String, SkillStatus>` - Per-skill status tracking

**Key Design Decision**: Using `Arc<dyn Skill>` enables:
- Shared ownership across multiple agents
- Thread-safe access without cloning
- Dynamic dispatch for heterogeneous skill types

### Registry Methods

#### register()
```rust
pub fn register(&mut self, skill: Arc<dyn Skill>) {
    let id = skill.id().to_string();
    self.statuses.insert(id.clone(), SkillStatus::Ready);
    self.skills.insert(id, skill);
}
```

**Behavior**: Overwrites existing skills with the same ID (per issue requirements). The new status is always `Ready` on registration.

#### get()
```rust
pub fn get(&self, id: &str) -> Option<Arc<dyn Skill>> {
    self.skills.get(id).cloned()
}
```

**Key Insight**: Since `Arc<T>` implements `Clone`, we can use `.cloned()` to return `Arc<dyn Skill>` from a reference. This is idiomatic for sharing owned data through references.

#### list()
```rust
pub fn list(&self) -> Vec<&str> {
    self.skills.keys().map(|s| s.as_str()).collect()
}
```

**Design Choice**: Returns `Vec<&str>` (references to owned strings in the HashMap) rather than cloning. This is efficient but means the returned strings are tied to the registry's lifetime.

#### assign_to_agent() / skills_for_agent()
```rust
pub fn skills_for_agent(&self, agent_id: &str) -> Vec<Arc<dyn Skill>> {
    self.agent_skills
        .get(agent_id)
        .map(|ids| ids.iter().filter_map(|id| self.skills.get(id).cloned()).collect())
        .unwrap_or_default()
}
```

**Key Behavior**: 
- Returns only skills that exist in the registry
- Silently skips unregistered skill IDs (doesn't fail)
- Returns empty vector if agent has no assignments

### Testing Considerations

#### Trait Object Comparison

A significant challenge was testing with `Arc<dyn Skill>`. The `Skill` trait requires `Debug` but not `PartialEq`. This meant direct comparison of `Arc<dyn Skill>` values in tests failed.

**Solution**: Verify behavior through the `id()` method instead:
```rust
// Wrong: doesn't compile (no PartialEq for dyn Skill)
assert_eq!(registry.get("skill-1"), Some(skill1));

// Correct: compare IDs
assert!(registry.get("skill-1").is_some());
assert_eq!(registry.get("skill-1").unwrap().id(), "skill-1");
```

#### Test Skill Implementation

The test implementation needed to derive `Debug` since `Skill` requires it:
```rust
#[derive(Debug)]
struct TestSkill {
    meta: SkillMeta,
    id: String,
}
```

## Files Created/Modified

1. **crates/aisopod-plugin/src/skills/registry.rs** - New file with complete implementation
2. **crates/aisopod-plugin/src/skills/mod.rs** - Added exports for `SkillRegistry` and `SkillStatus`

## Acceptance Criteria Met

- ✅ `SkillRegistry` struct is defined and publicly accessible
- ✅ `register()` adds a skill and sets its initial status to `Ready`
- ✅ `get()` returns a skill by ID
- ✅ `list()` returns all registered skill IDs
- ✅ `skills_for_agent()` returns the correct skills assigned to an agent
- ✅ `SkillStatus` enum supports `Ready`, `Degraded`, `Failed`, and `Unloaded` variants
- ✅ `status()` returns the current status for a given skill
- ✅ `cargo check -p aisopod-plugin` compiles without errors
- ✅ `cargo test -p aisopod-plugin` passes (122 tests)

## Future Considerations

1. **Thread Safety**: The current implementation is not thread-safe. Consider adding `Arc<Mutex<SkillRegistry>>` or using `Arc<RwLock<SkillRegistry>>` for concurrent access.

2. **Event Notifications**: Consider adding callbacks/notifications when skills are registered, unregistered, or change status.

3. **Skill Dependencies**: The registry might need to track dependency relationships between skills.

4. **Persistence**: Consider adding methods to save/load the registry state for persistence across restarts.

5. **Health Check Integration**: The `status()` method could be extended to perform actual health checks, not just return stored status.

## References

- Issue: #117
- Dependencies: Issue #116 (Skill trait, SkillMeta, SkillCategory)
- Related Issues: #118 (Skill Discovery), #119 (Skill-Agent Integration)
