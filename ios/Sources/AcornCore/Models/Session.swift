import Foundation
import GRDB

public struct Session: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var id: String
    public var rawInput: String
    public var aiSummary: String?
    public var language: String
    public var providerId: String?
    public var model: String?
    public var createdAt: Date
    public var completedAt: Date?

    public static let databaseTableName = "sessions"

    public enum CodingKeys: String, CodingKey {
        case id
        case rawInput = "raw_input"
        case aiSummary = "ai_summary"
        case language
        case providerId = "provider_id"
        case model
        case createdAt = "created_at"
        case completedAt = "completed_at"
    }

    public init(
        id: String = UUID().uuidString.lowercased(),
        rawInput: String,
        aiSummary: String? = nil,
        language: String,
        providerId: String? = nil,
        model: String? = nil,
        createdAt: Date = Date(),
        completedAt: Date? = nil
    ) {
        self.id = id
        self.rawInput = rawInput
        self.aiSummary = aiSummary
        self.language = language
        self.providerId = providerId
        self.model = model
        self.createdAt = createdAt
        self.completedAt = completedAt
    }
}
