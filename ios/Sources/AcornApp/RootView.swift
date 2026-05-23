// AcornApp — the application-level wiring layer.
//
// Once a sibling Xcode App target lands (future PR), its `@main` entry
// point will instantiate `RootView` from this module and inject the
// observable stores from `AcornCore`. Until then, this library compiles
// standalone via SPM so CI can build it.

import SwiftUI
import AcornCore
import AcornUI

/// Top-level Acorn iOS view.
///
/// Currently just hosts `AcornPlaceholderView`. In PR #2+ this becomes the
/// router root that switches between `InputView`, `StashView`, and
/// `SettingsView` (see ../../PLAN.md §2.5).
public struct RootView: View {
    public init() {}

    public var body: some View {
        AcornPlaceholderView()
    }
}

#Preview("Root") {
    RootView()
}
