import Foundation
import Observation
import AcornCore

@MainActor
@Observable
public final class ProvidersStore {
    public var catalog: [ProviderMetadata] = ProviderCatalog.all
    public var configs: [String: ProviderConfig] = [:]
    public var hasCredentials: [String: Bool] = [:]
    public var lastError: String?

    private let service: ProviderConfigService

    public init(service: ProviderConfigService) {
        self.service = service
    }

    public func hydrate() async {
        await reloadConfigs()
        await reloadCredentials()
    }

    public func reloadConfigs() async {
        if let list = try? await service.list() {
            configs = Dictionary(uniqueKeysWithValues: list.map { ($0.providerId, $0) })
        }
    }

    public func reloadCredentials() async {
        var nextMap: [String: Bool] = [:]
        for metadata in catalog where metadata.requiresApiKey {
            let hasIt = (try? await service.hasCredentials(providerId: metadata.id)) ?? false
            nextMap[metadata.id] = hasIt
        }
        hasCredentials = nextMap
    }

    public func saveCredentials(providerId: String, apiKey: String) async {
        do {
            try await service.saveCredentials(providerId: providerId, apiKey: apiKey)
            hasCredentials[providerId] = true
            _ = try await service.save(
                providerId: providerId,
                enabled: true,
                customEndpoint: configs[providerId]?.customEndpoint,
                selectedModel: configs[providerId]?.selectedModel
            )
            await reloadConfigs()
        } catch {
            lastError = "Could not save credentials: \(error.localizedDescription)"
        }
    }

    public func deleteCredentials(providerId: String) async {
        do {
            try await service.deleteCredentials(providerId: providerId)
            hasCredentials[providerId] = false
        } catch {
            lastError = "Could not delete credentials: \(error.localizedDescription)"
        }
    }

    public func selectModel(providerId: String, model: String?) async {
        do {
            let existing = configs[providerId]
            _ = try await service.save(
                providerId: providerId,
                enabled: existing?.enabled ?? true,
                customEndpoint: existing?.customEndpoint,
                selectedModel: model
            )
            await reloadConfigs()
        } catch {
            lastError = error.localizedDescription
        }
    }

    public func metadata(for providerId: String) -> ProviderMetadata? {
        catalog.first(where: { $0.id == providerId })
    }
}
