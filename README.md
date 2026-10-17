# PRC Display Studio

Native Rust framebuffer renderer and Windows WPF control/editor for the investigated Thermalright 320×240 LCD variant. Personal project, not an official Thermalright application. Compatibility with other models is not established.

## What is included
- USB HID transport and RGB565 conversion.
- Optional Windows telemetry and explicit missing-reading states.
- HUD renderer, theme loader and Windows theme editor.
- A neutral original background. Import your own 320×240 PNG/WebP/JPEG artwork.

Character artwork and portfolio screenshots are intentionally not distributed here. Dependencies retain their own licences. This source snapshot has no project-wide licence grant yet; contact the owner for redistribution terms.

## Build and verify
Windows with Rust stable and a C++ linker/build tools installed:
```powershell
cargo test --locked
cargo build --release --locked
cargo run --bin prc-lcd -- --list
cargo run --bin prc-lcd -- custom
powershell -NoProfile -File scripts/control-app.ps1
```
The first commands build/test/render previews. They do not write to a USB device. The native display action `prc-lcd --live custom` accesses the supported panel. Verify your USB model before using it. Unavailable sensors render a dash; no sensor bridge or elevated startup task is installed by this export.

## Create a theme
Put a background in `themes/my-theme/art.png` and a manifest alongside it. The editor can compose and save manifests. See [AI theme guide](AI-THEME-GUIDE.md) and the [example manifest](themes/example/theme.json). The example uses `art.png`; replace it with an image you have permission to use. The neutral asset is available at `assets/neutral.png`.

## Source boundary
Clean export from a private working project, with new Git history. Runtime logs, private configuration, personal machine data, binary build output and unrelated integrations are excluded. No claim of complete security certification is made.
