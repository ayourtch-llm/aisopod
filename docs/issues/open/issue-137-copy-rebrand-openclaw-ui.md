# Issue 137: Copy and Rebrand OpenClaw Lit UI to Aisopod

## Summary
Copy the existing Lit-based OpenClaw UI to the aisopod repository and rebrand all references from "OpenClaw" to "Aisopod", including component names, package metadata, and all source text.

## Location
- Crate: N/A (frontend)  
- File: `ui/` directory (copied from `tmp/openclaw/ui/`)

## Current Behavior
The aisopod repository has no web UI. The OpenClaw UI exists in `tmp/openclaw/ui/` with OpenClaw-specific branding, component names, and metadata.

## Expected Behavior
A fully rebranded Lit-based UI exists at `ui/` in the aisopod repo root. All references to "OpenClaw" and "openclaw" are replaced with "Aisopod" and "aisopod" respectively. Component tag names use the `aisopod-` prefix.

## Impact
This is the foundation for the entire Web Control UI feature (plan 0014). All subsequent UI issues depend on this copy-and-rebrand step being completed first.

## Suggested Implementation

1. **Copy the UI directory:**
   ```bash
   cp -r tmp/openclaw/ui/ ui/
   ```

2. **Rename component prefixes in all source files:**
   Use a recursive find-and-replace across `ui/src/`:
   ```bash
   # Replace tag names and references
   find ui/src/ -type f \( -name '*.ts' -o -name '*.html' -o -name '*.css' \) \
     -exec sed -i 's/openclaw-/aisopod-/g' {} +
   find ui/src/ -type f \( -name '*.ts' -o -name '*.html' -o -name '*.css' \) \
     -exec sed -i 's/OpenClaw/Aisopod/g' {} +
   find ui/src/ -type f \( -name '*.ts' -o -name '*.html' -o -name '*.css' \) \
     -exec sed -i 's/openclaw/aisopod/g' {} +
   ```

3. **Update `ui/package.json`:**
   ```json
   {
     "name": "aisopod-ui",
     "description": "Aisopod Web Control UI"
   }
   ```

4. **Rename any files that contain "openclaw" in their filename:**
   ```bash
   find ui/src/ -name '*openclaw*' | while read f; do
     mv "$f" "$(echo $f | sed 's/openclaw/aisopod/g')"
   done
   ```

5. **Update `index.html`** title and meta tags:
   ```html
   <title>Aisopod</title>
   ```

6. **Verify no remaining references:**
   ```bash
   grep -ri 'openclaw' ui/
   # Should return no results
   ```

## Dependencies
None (standalone UI task).

## Acceptance Criteria
- [ ] `ui/` directory exists at repo root with the full Lit UI source
- [ ] All "OpenClaw"/"openclaw" string references replaced with "Aisopod"/"aisopod"
- [ ] Component tag names use `<aisopod-*>` prefix (e.g., `<aisopod-app>`)
- [ ] `package.json` name and metadata updated
- [ ] `grep -ri 'openclaw' ui/` returns no results
- [ ] UI builds without errors (`pnpm install && pnpm build`)

---
*Created: 2026-02-15*
