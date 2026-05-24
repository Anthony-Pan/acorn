import Foundation

public struct ProviderInputs: Sendable {
    public let metadata: ProviderMetadata
    public let apiKey: String?
    public let model: String?
    public let customEndpoint: String?

    public init(
        metadata: ProviderMetadata,
        apiKey: String? = nil,
        model: String? = nil,
        customEndpoint: String? = nil
    ) {
        self.metadata = metadata
        self.apiKey = apiKey
        self.model = model
        self.customEndpoint = customEndpoint
    }
}

public enum ProviderFactory {
    public static func build(_ inputs: ProviderInputs) throws -> any Provider {
        switch inputs.metadata.apiFormat {
        case .anthropic:
            guard let key = inputs.apiKey, !key.isEmpty else {
                throw ProviderError.missingCredentials(inputs.metadata.id)
            }
            return AnthropicProvider(
                metadata: inputs.metadata,
                apiKey: key,
                model: inputs.model,
                customEndpoint: inputs.customEndpoint
            )
        case .openAiCompatible:
            guard let key = inputs.apiKey, !key.isEmpty else {
                throw ProviderError.missingCredentials(inputs.metadata.id)
            }
            return OpenAICompatibleProvider(
                metadata: inputs.metadata,
                apiKey: key,
                model: inputs.model,
                customEndpoint: inputs.customEndpoint
            )
        case .ollama:
            return UnsupportedProvider(
                metadata: inputs.metadata,
                reason: "Ollama doesn't run on iOS. Pick a cloud provider in Settings."
            )
        case .cli:
            return UnsupportedProvider(
                metadata: inputs.metadata,
                reason: "CLI providers aren't available on iOS."
            )
        case .gemini:
            throw ProviderError.notImplemented("Google Gemini provider lands in v1.1.")
        case .acornCloud:
            throw ProviderError.notImplemented("Acorn Cloud lands in v2.")
        case .foundationModels:
            #if canImport(FoundationModels)
            if #available(iOS 26.0, macOS 26.0, visionOS 26.0, *) {
                return FoundationModelsProvider(metadata: inputs.metadata)
            } else {
                throw ProviderError.platformUnsupported(
                    "Apple Intelligence requires iOS 26 or later. Pick a cloud provider in Settings."
                )
            }
            #else
            throw ProviderError.notImplemented(
                "FoundationModels framework not available in this toolchain."
            )
            #endif
        }
    }
}
