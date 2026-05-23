// AcornUI — SwiftUI views, design system, and animations for Acorn iOS.
//
// Depends on `AcornCore`. Where the macOS app uses Tailwind v4 + shadcn,
// this module re-implements the same warm autumn palette and component
// primitives natively in SwiftUI. iOS 26+ uses Liquid Glass material;
// earlier versions fall back to `.regularMaterial`.
//
// What lives here (incrementally landed — see ../../PLAN.md):
//   PR #6  — Design system: color tokens, typography, glass modifier
//   PR #7  — InputView (text editor, voice button, provider selector)
//   PR #8  — StashView with streaming task pop-in animation
//   PR #9  — SettingsView (provider sidebar + per-provider config)
//
// PR #1 (this PR) only carries the module anchor so dependent targets can
// import it. No real views yet.

import SwiftUI
import AcornCore

/// Placeholder view used by `AcornApp.RootView` until real screens land.
///
/// Deliberately tiny and self-contained so PR #1 demonstrates the
/// `AcornUI → AcornCore` dependency chain works without committing to any
/// design choices that PR #6 will re-litigate.
public struct AcornPlaceholderView: View {
    public init() {}

    public var body: some View {
        VStack(spacing: 12) {
            Text("🌰")
                .font(.system(size: 72))
                .accessibilityHidden(true)
            Text("Acorn")
                .font(.title.weight(.semibold))
            Text(acornCoreBanner())
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 24)
            Text("iOS scaffold landed. Real UI in PRs 6–9.")
                .font(.footnote)
                .foregroundStyle(.tertiary)
        }
        .padding()
    }
}

#Preview("Placeholder") {
    AcornPlaceholderView()
}
