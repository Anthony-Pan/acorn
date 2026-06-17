import Foundation
import Observation
import AcornCore

@MainActor
@Observable
public final class SettingsStore {
    public var activeProviderId: String = ProviderCatalog.defaultId
    public var theme: String = "system"
    public var language: String = SettingsStore.detectDefaultLanguage()
    public var speechLanguage: String? = nil
    public var sharedMemory: String = ""
    public var soundsMuted: Bool = false

    private let service: SettingsService
    private let memoryService: MemoryService

    public init(service: SettingsService, memory: MemoryService? = nil) {
        self.service = service
        self.memoryService = memory ?? MemoryService(settings: service)
    }

    public func hydrate() async {
        if let provider = try? await service.get(.activeProvider) {
            activeProviderId = provider
        }
        if let t = try? await service.get(.theme) {
            theme = t
        }
        if let l = try? await service.get(.language), !l.isEmpty {
            language = l
        }
        if let s = try? await service.get(.activeSpeechLanguage) {
            speechLanguage = s.isEmpty ? nil : s
        }
        sharedMemory = (try? await memoryService.read()) ?? ""
        if let muted = try? await service.get(.soundsMuted) {
            soundsMuted = muted == "1"
            Sounds.muted = soundsMuted
        }
    }

    public func setActiveProvider(_ providerId: String) async {
        activeProviderId = providerId
        try? await service.set(.activeProvider, value: providerId)
    }

    public func setTheme(_ value: String) async {
        theme = value
        try? await service.set(.theme, value: value)
    }

    public func setLanguage(_ code: String) async {
        language = code
        try? await service.set(.language, value: code)
    }

    public func setSpeechLanguage(_ identifier: String?) async {
        speechLanguage = (identifier?.isEmpty ?? true) ? nil : identifier
        if let id = identifier, !id.isEmpty {
            try? await service.set(.activeSpeechLanguage, value: id)
        } else {
            try? await service.remove(.activeSpeechLanguage)
        }
    }

    public func setSharedMemory(_ value: String) async {
        sharedMemory = value
        try? await memoryService.write(value)
    }

    public func setSoundsMuted(_ muted: Bool) async {
        soundsMuted = muted
        Sounds.muted = muted
        try? await service.set(.soundsMuted, value: muted ? "1" : "0")
    }

    public static func availableAILanguages() -> [(code: String, displayName: String)] {
        [
            ("en", "English"),
            ("zh", "简体中文"),
            ("ja", "日本語"),
            ("ko", "한국어"),
            ("es", "Español"),
            ("fr", "Français"),
            ("de", "Deutsch"),
        ]
    }

    private static func detectDefaultLanguage() -> String {
        let pref = Locale.preferredLanguages.first ?? Locale.current.identifier
        let code = pref.lowercased()
        for (id, _) in availableAILanguages() {
            if code.hasPrefix(id) { return id }
        }
        return "en"
    }
}
