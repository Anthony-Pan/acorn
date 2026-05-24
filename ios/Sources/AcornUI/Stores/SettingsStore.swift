import Foundation
import Observation
import AcornCore

@MainActor
@Observable
public final class SettingsStore {
    public var activeProviderId: String = ProviderCatalog.defaultId
    public var theme: String = "system"
    public var language: String = Locale.current.identifier

    private let service: SettingsService

    public init(service: SettingsService) {
        self.service = service
    }

    public func hydrate() async {
        if let provider = try? await service.get(.activeProvider) {
            activeProviderId = provider
        }
        if let t = try? await service.get(.theme) {
            theme = t
        }
        if let l = try? await service.get(.language) {
            language = l
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
}
