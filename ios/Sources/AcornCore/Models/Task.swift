import Foundation
import GRDB

public struct Task: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var id: String
    public var sessionId: String
    public var title: String
    public var description: String?
    public var durationMinutes: Int64
    public var priority: Priority
    public var orderIndex: Int64
    public var status: TaskStatus
    public var createdAt: Date
    public var startedAt: Date?
    public var completedAt: Date?

    public static let databaseTableName = "tasks"

    public enum CodingKeys: String, CodingKey {
        case id
        case sessionId = "session_id"
        case title
        case description
        case durationMinutes = "duration_minutes"
        case priority
        case orderIndex = "order_index"
        case status
        case createdAt = "created_at"
        case startedAt = "started_at"
        case completedAt = "completed_at"
    }

    public init(
        id: String = UUID().uuidString.lowercased(),
        sessionId: String,
        title: String,
        description: String? = nil,
        durationMinutes: Int64,
        priority: Priority,
        orderIndex: Int64,
        status: TaskStatus = .pending,
        createdAt: Date = Date(),
        startedAt: Date? = nil,
        completedAt: Date? = nil
    ) {
        self.id = id
        self.sessionId = sessionId
        self.title = title
        self.description = description
        self.durationMinutes = durationMinutes
        self.priority = priority
        self.orderIndex = orderIndex
        self.status = status
        self.createdAt = createdAt
        self.startedAt = startedAt
        self.completedAt = completedAt
    }
}
