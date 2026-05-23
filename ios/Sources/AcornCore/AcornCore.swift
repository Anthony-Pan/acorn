// AcornCore — pure Swift business logic for Acorn iOS.
//
// This module is intentionally **UI-free** (no SwiftUI, no UIKit). It is
// linkable from the main app, the WidgetKit extension, the Share Extension,
// and the App Intents target without dragging in the entire SwiftUI runtime.
//
// What lives here (incrementally landed across PRs — see ../../PLAN.md):
//   PR #2 — Models + GRDB database + migrations (mirrors src-tauri/migrations)
//   PR #3 — Session / Task / Subtask services (mirrors Tauri commands)
//   PR #4 — Provider protocol + Anthropic + OpenAI-compatible providers
//   PR #5 — decompose() AsyncThrowingStream with 120ms pop-in cadence
//   PR #10 — FoundationModelsProvider (iOS 26+ on-device LLM)
//
// PR #1 (this PR) only carries a version anchor so the module compiles and
// can be imported from sibling targets. No business logic yet.

import Foundation

/// The semantic version of the Acorn iOS app.
///
/// Kept in sync with the macOS Acorn version (see root `package.json`).
/// Bumped per the release process described in `../../PLAN.md`.
public enum AcornVersion {
    public static let current: String = "0.1.0-pre"
    public static let macOSEquivalent: String = "1.0.0"
}

/// Returns a one-line banner identifying the Acorn iOS core build.
///
/// Primarily used by smoke tests and the placeholder `RootView` until real
/// UI lands. Kept here (rather than in `AcornApp`) so the Widget and
/// AppIntents targets can surface the same banner without dragging
/// SwiftUI into their bundle.
public func acornCoreBanner() -> String {
    "🌰 Acorn iOS core \(AcornVersion.current) (macOS parity: \(AcornVersion.macOSEquivalent))"
}
