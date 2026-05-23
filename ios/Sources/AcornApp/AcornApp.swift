// AcornApp — App-level bootstrap.
//
// The `@main` App entry point is intentionally **not declared here** in
// PR #1: a pure-SPM library cannot host an iOS App target. A future PR
// adds an Xcode App project that imports this module, declares
// `@main struct AcornIOSApp: App`, and wires the Scene to `RootView()`.
//
// For now, `bootstrap()` is exposed as a no-op entry point so the eventual
// App target can call into AcornApp from its launch handler without
// AcornApp itself needing to know whether the host is the main app,
// a UI test, or a SwiftUI Preview.

import Foundation
import AcornCore

/// Bootstraps the Acorn iOS runtime.
///
/// In future PRs this initializes the database, hydrates stores,
/// registers App Intents, and primes the FoundationModels session.
/// Right now it just logs the core banner so a smoke test can prove
/// the module link graph works end-to-end.
@MainActor
public func bootstrap() {
    print(acornCoreBanner())
}
