// Smoke tests for AcornCore. Real test coverage lands with each subsequent
// PR (models in PR #2, services in PR #3, providers in PR #4–5, etc.).

import Testing
@testable import AcornCore

@Test("AcornVersion exposes a semver-shaped current version")
func acornVersionIsSemverShaped() {
    let parts = AcornVersion.current.split(separator: "-", maxSplits: 1).first?
        .split(separator: ".") ?? []
    #expect(parts.count == 3, "Expected MAJOR.MINOR.PATCH, got \(AcornVersion.current)")
    for part in parts {
        #expect(Int(part) != nil, "Version segment \(part) is not an integer")
    }
}

@Test("AcornVersion tracks the macOS equivalent for parity messaging")
func macOSEquivalentIsPresent() {
    #expect(!AcornVersion.macOSEquivalent.isEmpty)
}

@Test("acornCoreBanner mentions both Acorn and the macOS parity version")
func bannerMentionsBothVersions() {
    let banner = acornCoreBanner()
    #expect(banner.contains("Acorn"))
    #expect(banner.contains(AcornVersion.current))
    #expect(banner.contains(AcornVersion.macOSEquivalent))
}
