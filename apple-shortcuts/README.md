# Apple Shortcuts + Siri integration for Acorn

This directory ships the **Swift App Intents** package that lets users invoke
Acorn from Apple Shortcuts and Siri once the app is code-signed and notarised.

## Status

| Capability | Status |
| --- | --- |
| Source code (App Intents + AppShortcutsProvider) | ✅ Complete, in `Sources/AcornIntents/` |
| Wired into the main Tauri build | ❌ Pending (requires codesign — see `../docs/v1.1-shortcuts-build.md`) |
| Discoverable by Shortcuts.app | ❌ Pending |
| Discoverable by Siri | ❌ Pending |

Why the gap: the official Homebrew Cask and Apple Shortcuts both require the
app to be signed by a Developer ID and notarised. That is on the **v1.1
roadmap**. Until then, users can already invoke Acorn through deep links
(`acorn://stash?text=…`) using the Shortcuts "Open URL" action — see the
section below.

## Working today: Shortcuts via Open URL

Even without the Swift bundle compiled in, the `acorn://` URL scheme works the
moment the user has Acorn installed:

1. Open **Shortcuts.app**.
2. New Shortcut → **Add Action** → search "Open URL".
3. URL value:
   - `acorn://stash?text=Take%20the%20trash%20out` to stash a fixed string.
   - Combine with an **Ask Each Time** parameter to prompt for text.
   - `acorn://summon` to open the quick window.
   - `acorn://show` to bring the main window forward.
4. (Optional) Add the shortcut to the menu bar, Home Screen, or "Hey Siri,
   …" by configuring it in Shortcuts.app's settings.

Once Acorn is signed (v1.1), this directory's App Intents take over and the
"Open URL" indirection is no longer needed — Siri/Shortcuts will speak the
intents directly.

## What this package contains

`Sources/AcornIntents/AcornIntents.swift` defines:

- `StashIntent` — `Stash <text> in Acorn` with a multiline text parameter.
- `SummonIntent` — `Summon Acorn` to open the quick window.
- `OpenAcornIntent` — `Open Acorn` to focus the main window.
- `AcornShortcuts: AppShortcutsProvider` — registers the above as default
  shortcuts so Shortcuts.app discovers them automatically on first launch.
- `AcornDeepLinkLauncher` — opens the corresponding `acorn://` URL via
  `NSWorkspace`, so the Tauri Rust handler in `src-tauri/src/lib.rs`
  (`wire_deep_link`) does the heavy lifting.

This design intentionally **does not** open a second socket / XPC bridge into
the Tauri Rust runtime. Routing through the `acorn://` URL scheme keeps a
single source of truth for command dispatch (the Rust `wire_deep_link`
function) regardless of whether the trigger came from Shortcuts, Services,
the terminal, or a third-party launcher like Raycast.

## How to wire it in (v1.1, when signing lands)

Detailed walkthrough in `../docs/v1.1-shortcuts-build.md`. TL;DR:

1. `swift build -c release` produces `libAcornIntents.a`.
2. Add the static library and `AppIntents.framework` to the Tauri macOS
   bundle via `tauri.conf.json` → `bundle.macOS.frameworks` and a custom
   `build.rs` link step.
3. Sign the bundle with a Developer ID Application certificate, notarise
   through `notarytool`, and re-staple.
4. The first launch of the signed app registers the App Intents with the
   system. Shortcuts.app and `siri` find them within a few seconds.

## See also

- `../docs/v1.1-shortcuts-build.md` — full v1.1 build pipeline.
- `SERVICES.md` (this directory) — macOS Services menu integration plan.
- `../src-tauri/src/lib.rs` `wire_deep_link` — the Rust receiver.
