import Foundation
import GRDB

public struct Conversation: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var id: String
    public var title: String
    public var providerId: String?
    public var model: String?
    public var createdAt: Date
    public var lastMessageAt: Date

    public static let databaseTableName = "conversations"

    public enum CodingKeys: String, CodingKey {
        case id
        case title
        case providerId = "provider_id"
        case model
        case createdAt = "created_at"
        case lastMessageAt = "last_message_at"
    }

    public init(
        id: String = UUID().uuidString.lowercased(),
        title: String = "Untitled",
        providerId: String? = nil,
        model: String? = nil,
        createdAt: Date = Date(),
        lastMessageAt: Date = Date()
    ) {
        self.id = id
        self.title = title
        self.providerId = providerId
        self.model = model
        self.createdAt = createdAt
        self.lastMessageAt = lastMessageAt
    }
}
