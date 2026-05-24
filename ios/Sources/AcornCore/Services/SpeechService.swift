import Foundation

public actor SpeechService {
    private let providerConfigs: ProviderConfigService
    private let settings: SettingsService

    public init(providerConfigs: ProviderConfigService, settings: SettingsService) {
        self.providerConfigs = providerConfigs
        self.settings = settings
    }

    public nonisolated func listProviders() -> [SpeechProviderMetadata] {
        SpeechCatalog.all
    }

    public func resolveActive() async -> SpeechProviderMetadata {
        if let id = try? await settings.get(.activeSpeechProvider),
           let metadata = SpeechCatalog.get(id) {
            return metadata
        }
        return SpeechCatalog.get(SpeechCatalog.defaultId) ?? SpeechCatalog.all[0]
    }

    public func transcribe(
        audio: Data,
        mimeType: String,
        language: String?
    ) async throws -> TranscribeOutcome {
        let metadata = await resolveActive()
        let provider = try await makeProvider(metadata: metadata)
        let effectiveLanguage: String?
        if let language, !language.isEmpty {
            effectiveLanguage = language
        } else {
            effectiveLanguage = try? await settings.get(.activeSpeechLanguage)
        }
        return try await provider.transcribe(audio: audio, mimeType: mimeType, language: effectiveLanguage)
    }

    public func activeSpeechLanguage() async -> String? {
        try? await settings.get(.activeSpeechLanguage)
    }

    public func setActiveSpeechLanguage(_ identifier: String?) async throws {
        if let id = identifier, !id.isEmpty {
            try await settings.set(.activeSpeechLanguage, value: id)
        } else {
            try await settings.remove(.activeSpeechLanguage)
        }
    }

    private func makeProvider(metadata: SpeechProviderMetadata) async throws -> any SpeechProvider {
        switch metadata.id {
        case "system":
            #if canImport(Speech)
            return SFSpeechProvider(metadata: metadata)
            #else
            throw ProviderError.platformUnsupported(
                "Speech framework not available in this build."
            )
            #endif
        case "whisper_openai":
            let keyProviderId = metadata.apiKeyProviderId ?? "openai"
            guard let key = try await providerConfigs.readCredentials(providerId: keyProviderId),
                  !key.isEmpty else {
                throw ProviderError.missingCredentials(
                    "Whisper uses your OpenAI key — set it under Settings → OpenAI."
                )
            }
            return WhisperProvider(metadata: metadata, apiKey: key)
        default:
            throw ProviderError.notImplemented("Unknown speech provider \(metadata.id).")
        }
    }
}
