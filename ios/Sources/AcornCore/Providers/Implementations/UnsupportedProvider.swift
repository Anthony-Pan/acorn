import Foundation

public struct UnsupportedProvider: Provider {
    public let metadata: ProviderMetadata
    public let supportsTools: Bool = false
    public let reason: String

    public init(metadata: ProviderMetadata, reason: String) {
        self.metadata = metadata
        self.reason = reason
    }

    public func validateCredentials() async throws {
        throw ProviderError.platformUnsupported(reason)
    }

    public func decompose(_ request: DecomposeRequest) -> AsyncThrowingStream<DecomposeEvent, Error> {
        AsyncThrowingStream { continuation in
            continuation.finish(throwing: ProviderError.platformUnsupported(reason))
        }
    }

    public func chat(turns: [ChatTurn], systemPrompt: String, temperature: Double) async throws -> String {
        throw ProviderError.platformUnsupported(reason)
    }
}
