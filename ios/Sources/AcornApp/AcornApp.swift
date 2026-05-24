import Foundation
import SwiftUI
import AcornCore

@MainActor
public enum AcornBootstrap {
    public static func makeServices() async throws -> Services {
        try await Services.makeOnDisk()
    }
}

public func bootstrap() {
    print(acornCoreBanner())
}
