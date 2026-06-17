# Acorn iOS Port — Public Roadmap

> **Status:** Active. PR #1 (scaffold) is in flight.
> **Tech route:** Swift native (SwiftUI + Swift backend). On-device LLM via
> Apple FoundationModels (iOS 26+) with cloud-provider fallback.
> **Schema:** byte-compatible with the macOS Tauri app — see "Schema parity"
> in [`README.md`](README.md).

This document is the canonical, public version of the iOS port plan. A
longer working draft with raw research notes lives in `.sisyphus/plans/`
(gitignored agent workspace).

---

## 0. Goal

Rebuild Acorn's core value loop ("messy text dump → priority-ordered
task cards via streaming LLM") in iOS-native idioms, with **schema-level
compatibility** to the macOS desktop app so a future sync layer is
achievable without data migration pain.

We are **not** "porting" the desktop app pixel-for-pixel — that mental
model produces a worse product. The macOS-specific UX hooks (global
hotkey, menu bar accessory, quick window, Ollama) get **replaced** by
iOS-native equivalents (Siri, Widget, Live Activity, FoundationModels)
that the desktop app cannot have.

---

## 1. Locked decisions

| Decision | Choice | Rationale |
|---|---|---|
| UI toolkit | **SwiftUI** (iOS 18 floor; iOS 26 features behind `@available`) | Native-quality animation, Liquid Glass, App Intents, Widget all first-class |
| Backend language | **Swift** (no Rust on iOS) | Provider trait + DB layer re-implemented in Swift; reduces toolchain complexity |
| Local DB | **GRDB.swift** over SQLite | Mature, supports FTS5, schema-byte-identical to macOS sqlx side |
| Local LLM | **Apple FoundationModels** (iOS 26+) | Free, private, zero-latency; cloud providers fall back gracefully on older iOS |
| Cloud LLM | **Anthropic + 11-provider OpenAI-compatible catalog** | Same catalog as macOS, minus Ollama / CLI providers |
| Mac→iOS sync | **None in v1** | Schema kept identical so iCloud/CloudKit sync drops in cleanly later |
| Repo layout | **In-tree `ios/` directory** | Shared README/docs/CHANGELOG with macOS app; one source of truth |
| iOS-native superpowers | **Siri App Intent + Home Screen Widget + Lock Screen Live Activity + Share Extension** | Each replaces a macOS-only affordance the desktop app relies on |

---

## 2. What we keep / adapt / add / drop

### Keep 1:1 from macOS
- SQLite schema (8 tables, every column, every constraint, every index)
- 12-provider AI catalog (Anthropic, OpenAI, DeepSeek, OpenRouter, Groq, xAI, Mistral, Moonshot, Qwen, Zhipu, MiniMax, OpenAI-compat custom)
- `DecomposeRequest` / `DecomposeResponse` / `DecomposeEvent` wire shapes
- **120 ms task-card pop-in cadence** (the signature animation)
- Warm autumn palette + Geist Variable typography
- OpenAI Whisper cloud voice transcription

### Adapt to iOS-native
| macOS | iOS |
|---|---|
| `⌘⇧A` global shortcut | Siri App Intent ("Stash to Acorn: ...") |
| Menu bar accessory | Home Screen Widget + Lock Screen Widget |
| Quick window | Share Extension (receive from any app) + Spotlight |
| `keyring` (Rust) keychain | iOS Keychain Services (Security framework) |
| Tauri `Channel<T>` streaming | Swift `AsyncThrowingStream<T>` |

### Add (iOS-exclusive)
- **`FoundationModelsProvider`** — on-device LLM, zero cost, zero latency
- **App Intents** — Siri, Spotlight, Shortcuts.app, Action Button
- **WidgetKit** — Home + Lock screen
- **ActivityKit** — in-progress task on Lock Screen + Dynamic Island
- **`SFSpeechRecognizerProvider`** — fully on-device speech-to-text (the
  macOS app marks this "v1.1 coming soon"; iOS ships day one)
- **Share Extension** — text-in from any app

### Drop entirely (no iOS equivalent)
- Quick window (no floating windows on iOS)
- Global shortcut (no system-wide shortcuts on iOS)
- Menu bar / tray
- Ollama provider (won't run on phone)
- CLI providers (`claude_cli`, `codex_cli`, `gemini_cli`, `hermes_cli`)
- Window-management Tauri commands

---

## 3. Module architecture

```
ios/
├── Package.swift                        SPM manifest
└── Sources/
    ├── AcornCore/                       Pure logic — no UI imports
    │   ├── Models/                      Codable + GRDB records
    │   ├── Database/                    GRDB pool, migrator
    │   ├── Migrations/                  *.sql copied verbatim from src-tauri/
    │   ├── Providers/                   Provider protocol + impls
    │   ├── Speech/                      SpeechProvider protocol + impls
    │   ├── Keychain/                    iOS Keychain wrapper
    │   └── Utilities/
    ├── AcornUI/                         SwiftUI — depends on AcornCore
    │   ├── Views/{Input,Stash,Settings}/
    │   ├── Components/                  Reusable views
    │   ├── Design/                      Color + typography tokens
    │   └── Animations/                  120 ms pop-in & transitions
    ├── AcornApp/                        App wiring — RootView + bootstrap()
    └── AcornIntents/                    App Intents target (lands PR #12)
```

Plus future Xcode targets:
- `Widget/` — WidgetKit extension (PR #13)
- `ShareExtension/` — receive text from any app (PR #15)

**Strict separation:** AcornCore never imports SwiftUI / UIKit. This lets
the Widget, Share Extension, and App Intents bundles all link it without
pulling in the entire UI runtime.

---

## 4. PR sequence (~20 PRs over 5 months)

Each PR is **scope-bounded** — the "out of scope" line on each is
load-bearing. No PR mixes concerns.

### Phase 0 — Foundation
- **PR #1** *(this PR)* `feat(ios): scaffold ios/ project with launchable AcornApp`
  Directory structure, `Package.swift`, README, this PLAN, smoke tests,
  CI job (` swift build ` + ` swift test `), **plus** an Xcode App target
  managed by xcodegen (`App/project.yml`) so the scaffold actually
  launches on iPhone Simulator with a placeholder ` 🌰 Acorn ` screen.
  No business logic, no real screens.
- **PR #2** `feat(ios): GRDB schema + migrations + Codable models`
  Migrations copied verbatim from `src-tauri/migrations/`; CI gate that
  fails on schema drift between macOS and iOS.

### Phase 1 — Core data + cloud providers
- **PR #3** `feat(ios): Session/Task/Subtask services`
- **PR #4** `feat(ios): Provider protocol + Anthropic + OpenAI-compatible`
- **PR #5** `feat(ios): decompose() AsyncThrowingStream + 120 ms cadence`

### Phase 2 — UI MVP
- **PR #6** `feat(ios): design system (color tokens + Geist + Liquid Glass)`
- **PR #7** `feat(ios): InputView (text + voice + provider chip)`
- **PR #8** `feat(ios): StashView with streaming task pop-in animation`
- **PR #9** `feat(ios): SettingsView (provider sidebar + per-provider config)`

### Phase 3 — FoundationModels
- **PR #10** `feat(ios): FoundationModelsProvider with @Generable streaming`
- **PR #11** `feat(ios): default to FoundationModels on iOS 26+`

### Phase 4 — iOS-native superpowers
- **PR #12** `feat(ios): StashIntent + AcornShortcutsProvider (Siri)`
- **PR #13** `feat(ios): Home + Lock Screen Widget`
- **PR #14** `feat(ios): Live Activity for in-progress task`

### Phase 5 — Polish
- **PR #15** `feat(ios): Share Extension target`
- **PR #16** `feat(ios): Whisper + SFSpeechRecognizer providers`
- **PR #17** `feat(ios): conversation + chat (multi-turn)`

### Phase 6 — Pre-release
- **PR #18** `feat(ios): accessibility audit + Dynamic Type + VoiceOver`
- **PR #19** `chore(ios): App Store metadata + privacy nutrition labels`
- **PR #20** `chore(ios): TestFlight build + signing setup`

---

## 5. Definition of Done (v1.0 iOS)

- [ ] All 20 PRs landed on `main`
- [ ] `swift build` + `swift test` green in CI
- [ ] `xcodebuild` succeeds for the App target on iOS 18 + iOS 26 simulators
- [ ] Schema byte-equality with `src-tauri/migrations/` verified in CI
- [ ] Manual smoke test on iPhone 15 Pro (iOS 26 — FoundationModels path)
- [ ] Manual smoke test on iPhone 12 (iOS 18 — cloud-provider path)
- [ ] Manual smoke test on iPad Pro (universal layout)
- [ ] App accepted in TestFlight
- [ ] Privacy nutrition labels accurate
- [ ] Screenshots + App Store description ready
- [ ] `README.md` + `ARCHITECTURE.md` updated with iOS section
- [ ] `CHANGELOG.md` `[Unreleased]` cleared

---

## 6. Out of scope for v1 (deferred to v1.1+)

- iCloud / CloudKit sync between macOS and iOS instances
- FoundationModels tool calling (depends on Apple shipping the API)
- macOS Catalyst variant (someday — would eventually replace Tauri)
- Acorn Cloud sync (paid managed service, v2)
- Background fetch / background decompose
- Push notifications

---

## 7. Risks tracked at the project level

| Risk | Mitigation |
|---|---|
| FoundationModels API churns between iOS 26 betas | Pin to iOS 26.0 GA; cloud fallback fully working before relying on on-device |
| Apple Intelligence not on all "supported" devices (M1 iPad / iPhone 15 Pro+ only) | Availability detection wired in PR #10; cloud fallback graceful |
| Schema drift between macOS sqlx and iOS GRDB | CI gate in PR #2 compares migration files byte-for-byte |
| App Store rejection for AI-generated content disclosure | Privacy nutrition labels + in-app provider transparency (PR #19) |
| Widget memory limit (~30 MB) with FTS5 index loaded | Widget loads only `tasks` rows by `session_id`, never the FTS5 index (PR #13) |
| Concurrent DB writes from app + Widget + Live Activity | App Group + GRDB's WAL-aware DatabasePool; integration test in PR #14 |

---

## 8. Pointers

- macOS architecture reference: [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- Schema source of truth: [`../src-tauri/migrations/`](../src-tauri/migrations/)
- AI provider catalog source: [`../src-tauri/src/ai/metadata.rs`](../src-tauri/src/ai/metadata.rs)
- Existing Swift package style: [`../apple-shortcuts/`](../apple-shortcuts/)
- Agent contributor rules: [`../AGENTS.md`](../AGENTS.md) (gitignored, dev-local)
