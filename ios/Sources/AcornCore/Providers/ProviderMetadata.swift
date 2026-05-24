import Foundation

public enum ApiFormat: String, Codable, Sendable {
    case anthropic
    case openAiCompatible = "openai_compatible"
    case ollama
    case gemini
    case acornCloud = "acorn_cloud"
    case cli
    case foundationModels = "foundation_models"
}

public enum ProviderCategory: String, Codable, Sendable {
    case recommended
    case local
    case advanced
    case comingSoon = "coming_soon"
}

public enum ProviderStatus: String, Codable, Sendable {
    case available
    case comingSoon = "coming_soon"
    case platformUnsupported = "platform_unsupported"
}

public struct ProviderMetadata: Sendable, Identifiable, Equatable {
    public let id: String
    public let displayName: String
    public let category: ProviderCategory
    public let apiFormat: ApiFormat
    public let defaultModel: String
    public let availableModels: [String]
    public let defaultEndpoint: String
    public let allowCustomEndpoint: Bool
    public let requiresApiKey: Bool
    public let featured: Bool
    public let status: ProviderStatus

    public init(
        id: String,
        displayName: String,
        category: ProviderCategory,
        apiFormat: ApiFormat,
        defaultModel: String,
        availableModels: [String],
        defaultEndpoint: String,
        allowCustomEndpoint: Bool = false,
        requiresApiKey: Bool = true,
        featured: Bool = false,
        status: ProviderStatus = .available
    ) {
        self.id = id
        self.displayName = displayName
        self.category = category
        self.apiFormat = apiFormat
        self.defaultModel = defaultModel
        self.availableModels = availableModels
        self.defaultEndpoint = defaultEndpoint
        self.allowCustomEndpoint = allowCustomEndpoint
        self.requiresApiKey = requiresApiKey
        self.featured = featured
        self.status = status
    }
}
