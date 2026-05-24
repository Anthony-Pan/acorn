import Foundation
import GRDB

public struct SpeechProviderConfig: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var providerId: String
    public var enabled: Bool
    public var selectedLanguage: String?
    public var lastUsedAt: Date?

    public var id: String { providerId }

    public static let databaseTableName = "speech_provider_configs"

    public enum CodingKeys: String, CodingKey {
        case providerId = "provider_id"
        case enabled
        case selectedLanguage = "selected_language"
        case lastUsedAt = "last_used_at"
    }

    public init(
        providerId: String,
        enabled: Bool = true,
        selectedLanguage: String? = nil,
        lastUsedAt: Date? = nil
    ) {
        self.providerId = providerId
        self.enabled = enabled
        self.selectedLanguage = selectedLanguage
        self.lastUsedAt = lastUsedAt
    }
}
