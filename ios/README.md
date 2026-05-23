# Acorn iOS

The iOS-native sibling of the [Acorn macOS desktop app](../README.md). Same
warm autumn aesthetic, same model-agnostic AI layer, same "stash your day
into focused task cards" core loop — re-built in SwiftUI with iOS-native
superpowers (Siri, Widgets, Live Activity, on-device LLM via Apple
FoundationModels) that the desktop app cannot have.

> **Status: scaffold (PR #1 of ~20).** This directory currently contains
> only the package manifest and module skeleton. There is no runnable app
> yet. See [`PLAN.md`](PLAN.md) for the full delivery roadmap.

## Layout

```
ios/
├── PLAN.md                  Multi-PR roadmap (this is the spec)
├── Package.swift            Pure SPM manifest — 3 libraries, no .xcodeproj
├── Sources/
│   ├── AcornCore/           Pure-Swift business logic (models, DB, providers)
│   ├── AcornUI/             SwiftUI views, design system, animations
│   └── AcornApp/            App-level wiring (RootView, bootstrap)
└── Tests/
    └── AcornCoreTests/      Swift Testing
```

`AcornCore` is intentionally **UI-free** so the future Widget, Share
Extension, and App Intents targets can all link it without dragging in the
SwiftUI runtime.

## Develop

Requirements: macOS 14+, Xcode 26+, Swift 6+.

```sh
cd ios
swift build           # builds all three libraries against the macOS toolchain
swift test            # runs the smoke tests
open Package.swift    # opens the package in Xcode (use Xcode 26+)
```

When the iOS App target lands (future PR) you will additionally need:

- An Apple Developer account
- An iOS 18 / iOS 26 simulator (or device) for `xcodebuild`
- Provisioning profile for the bundle identifier `app.acorn.ios`
  (matching the macOS app's `app.acorn.desktop` keychain service prefix)

## Schema parity with macOS

The most important non-obvious invariant: **`ios/Sources/AcornCore/Migrations/*.sql`
must be byte-identical to `../src-tauri/migrations/*.sql`.** Both apps share
the same SQLite schema so a future iCloud / Acorn-Cloud sync layer can be
dropped in without a one-off data migration. PR #2 will add CI that fails
on drift.

API keys live in the iOS Keychain under the service name
`app.acorn.desktop` (matching the macOS app), so a future iCloud Keychain
sync would Just Work.

## Why pure SPM in PR #1?

Two reasons:

1. **Reviewability.** A 12-line `Package.swift` plus a handful of source
   files is reviewable in a single pass. An `.xcodeproj` is a directory of
   plist/XML that generates merge conflicts on every PR.
2. **CI runs anywhere.** `swift build` works on a macOS GitHub Actions
   runner with no simulator boot. The App target (which does need
   `xcodebuild` + a simulator) lands in a later PR when there is actual
   app code to launch.

A future PR (~PR #2 or #3) adds either an Xcode `.xcodeproj` (committed)
or an XcodeGen / Tuist spec (preferred). Either path is fine — that
decision is captured in `PLAN.md` §1.1.

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
