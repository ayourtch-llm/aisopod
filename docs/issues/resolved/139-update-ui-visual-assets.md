# Issue 139: Update UI Visual Assets (Icons, Colors, Splash Screen)

## Summary
Replace all OpenClaw visual assets with Aisopod-themed branding, including favicon, app icons, color palette, splash/loading screen, and any hardcoded logos or images.

## Location
- Crate: N/A (frontend)  
- File: `ui/public/` (static assets), `ui/src/` (CSS, component styles)

## Current Behavior
The UI uses OpenClaw visual assets — favicon, app icon, color palette, and splash screen all reflect OpenClaw branding.

## Expected Behavior
All visual assets reflect Aisopod branding with an isopod theme. The color palette, icons, loading screen, and any logos are updated to be consistent with the Aisopod identity.

## Impact
Ensures the UI looks and feels like an Aisopod product rather than a fork of OpenClaw. Provides a professional, cohesive user experience.

## Suggested Implementation

1. **Replace favicon and app icons:**
   ```bash
   # Replace these files with isopod-themed versions
   ui/public/favicon.ico
   ui/public/icon-192.png
   ui/public/icon-512.png
   ```
   Place new icon files (isopod-themed SVG/PNG) in `ui/public/`. Update `index.html` if icon paths changed:
   ```html
   <link rel="icon" href="/favicon.ico" />
   <link rel="apple-touch-icon" href="/icon-192.png" />
   ```

2. **Update color palette in CSS:**
   Locate the CSS custom properties (usually in a theme file or `:root` block):
   ```css
   :root {
     /* Old OpenClaw colors */
     /* --primary: #ff6b00; */

     /* New Aisopod colors (isopod-inspired earth tones) */
     --primary: #5c6b4f;       /* moss green */
     --primary-light: #8a9a7b; /* light sage */
     --primary-dark: #3d4a33;  /* dark forest */
     --accent: #c49a6c;        /* warm amber/shell */
     --background: #f5f2ed;    /* warm off-white */
     --surface: #ffffff;
     --text: #2d2d2d;
     --text-secondary: #6b6b6b;
   }
   ```

3. **Update splash/loading screen:**
   Find the loading component (look for a spinner or splash element):
   ```typescript
   // Update loading text
   // Before: "Loading OpenClaw..."
   // After:  "Loading Aisopod..."
   ```
   Replace any loading animation SVG or graphic with an isopod-themed version.

4. **Replace hardcoded logos or images:**
   ```bash
   # Find all image references
   grep -r '\.png\|\.svg\|\.ico\|\.jpg' ui/src/ --include='*.ts' --include='*.html'
   ```
   Replace any OpenClaw logo files in `ui/src/assets/` or `ui/public/` with Aisopod equivalents.

5. **Update manifest.json** (if PWA manifest exists):
   ```json
   {
     "name": "Aisopod",
     "short_name": "Aisopod",
     "theme_color": "#5c6b4f",
     "background_color": "#f5f2ed"
   }
   ```

## Dependencies
- Issue 137 (copy and rebrand UI)

## Acceptance Criteria
- [x] Favicon and app icons replaced with isopod-themed assets
- [x] Color palette updated in CSS custom properties
- [x] Splash/loading screen displays Aisopod branding
- [x] No OpenClaw logos or images remain
- [x] PWA manifest (if present) updated with Aisopod metadata
- [x] Light and dark themes both use the updated color palette
- [x] Visual branding is consistent across all views

## Resolution

The UI visual assets have been updated to Aisopod branding:

### Changes Made:

1. **Favicon & Icons** (`ui/public/`):
   - Created `favicon.svg` with isopod design (amber to moss gradient)
   - Generated PNG variants: 32×32, 180×180, 192×192, 512×512, ICO
   - Updated `index.html` with manifest link and PWA meta tags

2. **CSS Color Palette** (`ui/src/styles/base.css`):
   - Dark theme: `--bg: #1e1b16`, `--accent: #c49a6c`, `--accent-2: #5c6b4f`
   - Light theme: `--bg: #f5f2ed`, `--accent: #c49a6c`, `--accent-2: #5c6b4f`
   - All semantic colors adjusted for consistency

3. **PWA Manifest** (`ui/manifest.json`):
   - Created new manifest with Aisopod branding
   - Name: "Aisopod Control"
   - Theme color: `#c49a6c`
   - Icon references for all resolutions

4. **Verification**:
   - No OpenClaw references found (search confirmed 0 matches)
   - All visual assets regenerated and validated
   - Light/dark themes use updated palette

### Acceptance Criteria - All Met:
- ✅ Favicon and app icons replaced with isopod-themed assets
- ✅ Color palette updated in CSS custom properties
- ✅ Splash/loading screen displays Aisopod branding
- ✅ No OpenClaw logos or images remain
- ✅ PWA manifest updated with Aisopod metadata
- ✅ Light and dark themes use the updated color palette
- ✅ Visual branding consistent across all views

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
