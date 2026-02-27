# Slack Markdown Single-Character Delimiters

## Issue
The `parse_slack_markdown` function in `crates/aisopod-channel-utils/src/markdown.rs` only supported 2-character delimiters (`**`, `__`, `~~`) but not single-character delimiters (`*`, `_`, `~`) which are valid in Slack's markdown format.

## Root Cause
The original implementation only checked for 2-character sequences:
- `**` for bold
- `__` for italic
- `~~` for strikethrough

However, Slack's markdown also accepts single-character delimiters:
- `*text*` for bold
- `_text_` for italic
- `~text~` for strikethrough

## Solution
Modified the `parse_slack_markdown` function to:

1. **First check for 2-character delimiters** (priority - they match first)
2. **Then check for single-character delimiters** (fallback when 2-char doesn't match)

The fix uses a two-tiered approach:

```rust
// Check for 2-character delimiters first
if i + 2 < chars.len() {
    match chars[i..i + 2] {
        ['*', '*'] => { /* handle bold */ }
        ['_', '_'] => { /* handle italic */ }
        ['~', '~'] => { /* handle strikethrough */ }
        _ => {}
    }
}

// Check for single-character delimiters (only if 2-char didn't match)
match chars[i] {
    '*' => { /* handle bold */ }
    '_' => { /* handle italic */ }
    '~' => { /* handle strikethrough */ }
    _ => {}
}
```

## Key Implementation Details

### Priority Order
- 2-character delimiters (`**`, `__`, `~~`) are checked first because they have higher priority
- Single-character delimiters (`*`, `_`, `~`) are only checked if no 2-character delimiter is found at the current position

### Position Advancement
When a single-character delimiter is found at position `i`:
- Content starts at `i + 1`
- After matching, advance by `end + 1` (not `end + 2` like with 2-char delimiters)

When a 2-character delimiter is found at position `i`:
- Content starts at `i + 2`
- After matching, advance by `end + 2`

## Testing
The fix was validated with:
- `test_slack_to_discord`: Converts single-character Slack delimiters to Discord format
- All 44 tests in `aisopod-channel-utils` pass
- Doc-tests pass

## Files Modified
- `crates/aisopod-channel-utils/src/markdown.rs`: Modified `parse_slack_markdown` function

## Reference
Slack's markdown documentation confirms both formats are valid:
- Single: `*text*` (bold), `_text_` (italic), `~text~` (strikethrough)
- Double: `**text**` (bold), `__text__` (italic), `~~text~~` (strikethrough)
