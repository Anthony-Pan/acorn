import Foundation

public enum ProviderError: Error, LocalizedError, Sendable {
    case invalidKey
    case rateLimited
    case network(String)
    case invalidResponse(String)
    case providerResponse(String)
    case ollamaNotRunning(String)
    case notImplemented(String)
    case keychain(String)
    case missingCredentials(String)
    case conversationLocked(lockedTo: String, attempted: String)
    case channelClosed
    case modelUnavailable(reason: String)
    case platformUnsupported(String)

    public var errorDescription: String? {
        switch self {
        case .invalidKey:
            return "Invalid API key for this provider."
        case .rateLimited:
            return "Rate-limited by the provider. Try again in a moment."
        case .network(let msg):
            return "Network error: \(msg)"
        case .invalidResponse(let msg):
            return "Invalid response from the model: \(msg)"
        case .providerResponse(let msg):
            return "Provider returned error: \(msg)"
        case .ollamaNotRunning(let url):
            return "Ollama is not running at \(url). Start it with `ollama serve`."
        case .notImplemented(let msg):
            return "Not implemented: \(msg)"
        case .keychain(let msg):
            return "Keychain error: \(msg)"
        case .missingCredentials(let id):
            return "No saved credentials for provider '\(id)'."
        case .conversationLocked(let lockedTo, let attempted):
            return "Conversation is locked to '\(lockedTo)'; cannot send via '\(attempted)'."
        case .channelClosed:
            return "Streaming channel closed unexpectedly."
        case .modelUnavailable(let reason):
            return "Model unavailable: \(reason)"
        case .platformUnsupported(let msg):
            return "Not supported on this platform: \(msg)"
        }
    }
}

public protocol Provider: Sendable {
    var metadata: ProviderMetadata { get }
    var supportsTools: Bool { get }
    func decompose(_ request: DecomposeRequest) -> AsyncThrowingStream<DecomposeEvent, Error>
    func validateCredentials() async throws
    func chat(turns: [ChatTurn], systemPrompt: String, temperature: Double) async throws -> String
}

public extension Provider {
    var supportsTools: Bool { true }
}
