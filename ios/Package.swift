// swift-tools-version: 6.0
import PackageDescription

// Acorn iOS — Swift package manifest.
//
// Three static libraries with strict-concurrency Swift 6:
//   AcornCore — pure business logic (DB, providers, models). No UI imports.
//                Linkable from Widget / ShareExtension / AppIntents targets.
//   AcornUI   — SwiftUI views, design system, animations.
//   AcornApp  — App-level wiring (RootView + bootstrap).
//
// External dependencies are minimized: GRDB.swift (SQLite ORM, mature, FTS5).
// Everything else is Apple frameworks (URLSession, Security, SwiftUI, etc.).

let package = Package(
    name: "AcornIOS",
    platforms: [
        .iOS(.v18),
        .macOS(.v14),
    ],
    products: [
        .library(name: "AcornCore", type: .static, targets: ["AcornCore"]),
        .library(name: "AcornUI", type: .static, targets: ["AcornUI"]),
        .library(name: "AcornApp", type: .static, targets: ["AcornApp"]),
    ],
    dependencies: [
        // Local pinned checkout of groue/GRDB.swift v7.9.0 to bypass slow
        // SwiftPM network resolution. The pinned version is committed in
        // the cache location below; contributors run the bootstrap script
        // in ios/README.md to materialize it on first clone.
        // Once SwiftPM mirror infra is in place we can switch back to:
        //   .package(url: "https://github.com/groue/GRDB.swift.git", exact: "7.9.0"),
        .package(name: "GRDB.swift", path: "../../.deps-cache/GRDB.swift"),
    ],
    targets: [
        .target(
            name: "AcornCore",
            dependencies: [
                .product(name: "GRDB", package: "GRDB.swift"),
            ],
            path: "Sources/AcornCore",
            resources: [
                .process("Migrations"),
            ],
            swiftSettings: strictConcurrency
        ),
        .target(
            name: "AcornUI",
            dependencies: ["AcornCore"],
            path: "Sources/AcornUI",
            swiftSettings: strictConcurrency
        ),
        .target(
            name: "AcornApp",
            dependencies: ["AcornCore", "AcornUI"],
            path: "Sources/AcornApp",
            swiftSettings: strictConcurrency
        ),
        .testTarget(
            name: "AcornCoreTests",
            dependencies: ["AcornCore"],
            path: "Tests/AcornCoreTests",
            swiftSettings: strictConcurrency
        ),
    ]
)

let strictConcurrency: [SwiftSetting] = [
    .swiftLanguageMode(.v6),
]
