// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "AcornIntents",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "AcornIntents",
            type: .static,
            targets: ["AcornIntents"]
        )
    ],
    targets: [
        .target(
            name: "AcornIntents",
            path: "Sources/AcornIntents"
        )
    ]
)
