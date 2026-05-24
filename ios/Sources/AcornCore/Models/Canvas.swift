import Foundation
import GRDB

public struct Canvas: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var id: String
    public var title: String
    public var content: String
    public var createdAt: Date
    public var updatedAt: Date

    public static let databaseTableName = "canvases"

    public enum CodingKeys: String, CodingKey {
        case id
        case title
        case content
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }

    public init(
        id: String = UUID().uuidString.lowercased(),
        title: String = "Untitled canvas",
        content: String = "",
        createdAt: Date = Date(),
        updatedAt: Date = Date()
    ) {
        self.id = id
        self.title = title
        self.content = content
        self.createdAt = createdAt
        self.updatedAt = updatedAt
    }
}
