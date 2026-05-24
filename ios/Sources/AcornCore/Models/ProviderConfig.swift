import Foundation
import GRDB

public struct ProviderConfig: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var providerId: String
    public var enabled: Bool
    public var customEndpoint: String?
    public var selectedModel: String?
    public var lastUsedAt: Date?

    public var id: String { providerId }

    public static let databaseTableName = "provider_configs"

    public enum CodingKeys: String, CodingKey {
        case providerId = "provider_id"
        case enabled
        case customEndpoint = "custom_endpoint"
        case selectedModel = "selected_model"
        case lastUsedAt = "last_used_at"
    }

    public init(
        providerId: String,
        enabled: Bool = true,
        customEndpoint: String? = nil,
        selectedModel: String? = nil,
        lastUsedAt: Date? = nil
    ) {
        self.providerId = providerId
        self.enabled = enabled
        self.customEndpoint = customEndpoint
        self.selectedModel = selectedModel
        self.lastUsedAt = lastUsedAt
    }
}
