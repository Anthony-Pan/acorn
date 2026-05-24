import Foundation

public enum SpeechProviderCategory: String, Codable, Sendable {
    case cloud
    case onDevice = "on_device"
}

public struct SpeechProviderMetadata: Sendable, Identifiable, Equatable {
    public let id: String
    public let displayName: String
    public let category: SpeechProviderCategory
    public let requiresApiKey: Bool
    public let apiKeyProviderId: String?
    public let status: ProviderStatus

    public init(
        id: String,
        displayName: String,
        category: SpeechProviderCategory,
        requiresApiKey: Bool,
        apiKeyProviderId: String? = nil,
        status: ProviderStatus = .available
    ) {
        self.id = id
        self.displayName = displayName
        self.category = category
        self.requiresApiKey = requiresApiKey
        self.apiKeyProviderId = apiKeyProviderId
        self.status = status
    }
}

public struct TranscribeOutcome: Sendable {
    public let text: String
    public let providerId: String

    public init(text: String, providerId: String) {
        self.text = text
        self.providerId = providerId
    }
}

public protocol SpeechProvider: Sendable {
    var metadata: SpeechProviderMetadata { get }
    func transcribe(audio: Data, mimeType: String, language: String?) async throws -> TranscribeOutcome
}

public enum SpeechCatalog {
    public static let all: [SpeechProviderMetadata] = [
        SpeechProviderMetadata(
            id: "system",
            displayName: "On-device (SFSpeechRecognizer)",
            category: .onDevice,
            requiresApiKey: false
        ),
        SpeechProviderMetadata(
            id: "whisper_openai",
            displayName: "OpenAI Whisper (cloud)",
            category: .cloud,
            requiresApiKey: true,
            apiKeyProviderId: "openai"
        ),
    ]

    public static var defaultId: String { "system" }

    public static func get(_ id: String) -> SpeechProviderMetadata? {
        all.first { $0.id == id }
    }
}
