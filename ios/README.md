# Acorn iOS

The iOS-native sibling of the [Acorn macOS desktop app](../README.md). Same
warm autumn aesthetic, same model-agnostic AI layer, same "stash your day
into focused task cards" core loop — re-built in SwiftUI with iOS-native
superpowers (Siri, Widgets, Live Activity, on-device LLM via Apple
FoundationModels) that the desktop app cannot have.

> **Status: scaffold (PR #1 of ~20).** Launchable on iPhone Simulator but
> contains only placeholder UI — real screens, DB, and providers land in
> PRs #2–#9. See [`PLAN.md`](PLAN.md) for the full roadmap.

## Layout

```
ios/
├── PLAN.md                  Multi-PR roadmap (this is the spec)
├── Package.swift            SPM manifest — 3 libraries
├── Sources/
│   ├── AcornCore/           Pure-Swift business logic (models, DB, providers)
│   ├── AcornUI/             SwiftUI views, design system, animations
│   └── AcornApp/            App-level wiring (RootView, bootstrap)
├── Tests/
│   └── AcornCoreTests/      Swift Testing
└── App/                     Xcode iOS App target (xcodegen-managed)
    ├── project.yml          Source of truth — regenerate .xcodeproj from this
    └── AcornIOS/            @main, Info.plist props, AppIcon, AccentColor
```

`AcornCore` is intentionally **UI-free** so the future Widget, Share
Extension, and App Intents targets can all link it without dragging in the
SwiftUI runtime.

## Requirements

- macOS 14+, Xcode 26+, Swift 6+
- [xcodegen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`) — generates the `.xcodeproj` from `App/project.yml`
- An iOS 18 or iOS 26 simulator (Xcode → Settings → Platforms)

## Quick start — launch on iPhone Simulator

```sh
cd ios/App
xcodegen generate                 # produces AcornIOS.xcodeproj (gitignored)
open AcornIOS.xcodeproj           # opens in Xcode
# In Xcode: pick "iPhone 17 Pro" (or any iOS 18+ simulator) → ⌘R
```

You should see a placeholder `🌰 Acorn` screen. That confirms the
module link graph (`AcornApp → AcornUI → AcornCore`) works end-to-end.

## Library-only workflow (no Xcode App target)

For PRs that touch only `AcornCore` / `AcornUI` / `AcornApp` libraries,
the App target isn't needed — `swift build` on macOS suffices:

```sh
cd ios
swift build           # builds all three libraries against the macOS toolchain
swift test            # runs the Swift Testing smoke tests
open Package.swift    # opens the package in Xcode for preview / debug
```

This is also exactly what CI does — see `.github/workflows/ci.yml` `iOS` job.

## Schema parity with macOS

The most important non-obvious invariant: **`ios/Sources/AcornCore/Migrations/*.sql`
must be byte-identical to `../src-tauri/migrations/*.sql`.** Both apps share
the same SQLite schema so a future iCloud / Acorn-Cloud sync layer can be
dropped in without a one-off data migration. PR #2 will add CI that fails
on drift.

API keys live in the iOS Keychain under the service name
`app.acorn.desktop` (matching the macOS app), so a future iCloud Keychain
sync would Just Work.

## Why xcodegen (not a committed .xcodeproj)?

The `.xcodeproj` is **not committed** — it's generated from
[`App/project.yml`](App/project.yml) every time you run `xcodegen`. This
trade is deliberate:

- ✅ The 70-line `project.yml` is readable and reviewable; a committed
  `.pbxproj` would be hundreds of lines of plist with UUIDs that change
  on every Xcode setting tweak, generating noisy diffs.
- ✅ Two devs on different Xcode versions produce identical projects
  because they regenerate from the same spec.
- ✅ The `Info.plist` is also generated from inline properties in
  `project.yml`, so bundle metadata stays in one place.
- ⚠️ Cost: contributors must `brew install xcodegen` once. Documented above.

This matches the pattern used by mature multi-target Swift codebases
(e.g. Kickstarter iOS, ProtonMail, several Apple sample apps).

## Relationship to the macOS app

The Tauri macOS app in `../src-tauri/` + `../src/` is **not going away**.
It remains the canonical macOS path for the foreseeable future. iOS and
macOS coexist with independent local databases in v1; sync arrives in a
later milestone (see `PLAN.md` §7).

## See also

- [`PLAN.md`](PLAN.md) — the canonical iOS port roadmap
- [`../AGENTS.md`](../AGENTS.md) — agent / contributor hard rules
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — Tauri/Rust/React architecture (the source we're mirroring)
- [`../apple-shortcuts/`](../apple-shortcuts/) — the existing Swift App Intents package for the macOS app
