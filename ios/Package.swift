// swift-tools-version: 6.0
import PackageDescription

// Acorn iOS — Swift package manifest.
//
// This package defines the three core libraries that ship in the Acorn iOS
// app (`AcornCore`, `AcornUI`, `AcornApp`). The actual iOS App target (with
// `@main` and an Info.plist) lives in a future PR that adds an Xcode App
// Project; this manifest is intentionally library-only so it can be opened
// in Xcode, built with `swift build` on macOS, and verified in CI without
// requiring the App Store toolchain.
//
// See `PLAN.md` for the full iOS port roadmap and PR sequence.

let package = Package(
    name: "AcornIOS",
    platforms: [
        // iOS 18 floor for general device coverage; FoundationModelsProvider
        // gates its own code behind `@available(iOS 26.0, *)` and degrades
        // to cloud providers below that.
        .iOS(.v18),
        // macOS 14 lets the core libraries compile in CI on a macOS runner
        // without needing an iOS simulator boot, and keeps the door open
        // for a future Catalyst variant.
        .macOS(.v14),
    ],
    products: [
        .library(name: "AcornCore", type: .static, targets: ["AcornCore"]),
        .library(name: "AcornUI", type: .static, targets: ["AcornUI"]),
        .library(name: "AcornApp", type: .static, targets: ["AcornApp"]),
    ],
    targets: [
        // AcornCore — pure business logic. No SwiftUI / UIKit imports.
        // Will host: models, GRDB wrappers, Provider protocol + impls,
        // Keychain wrapper, services. Linkable from Widget / ShareExtension
        // / AppIntents targets too.
        .target(
            name: "AcornCore",
            path: "Sources/AcornCore",
            swiftSettings: strictConcurrency
        ),
        // AcornUI — SwiftUI views, design system, animations.
        .target(
            name: "AcornUI",
            dependencies: ["AcornCore"],
            path: "Sources/AcornUI",
            swiftSettings: strictConcurrency
        ),
        // AcornApp — the App-level wiring (@main lives in a sibling Xcode
        // target eventually; this library exposes a `bootstrap()` entry
        // point and the top-level `RootView`).
        .target(
            name: "AcornApp",
            dependencies: ["AcornCore", "AcornUI"],
            path: "Sources/AcornApp",
            swiftSettings: strictConcurrency
        ),
        // Tests use Swift Testing (`import Testing`).
        .testTarget(
            name: "AcornCoreTests",
            dependencies: ["AcornCore"],
            path: "Tests/AcornCoreTests",
            swiftSettings: strictConcurrency
        ),
    ]
)

// Swift 6 strict concurrency — every target opts in so we catch data races
// at the boundary in PR #1, not in PR #20.
let strictConcurrency: [SwiftSetting] = [
    .swiftLanguageMode(.v6),
]
