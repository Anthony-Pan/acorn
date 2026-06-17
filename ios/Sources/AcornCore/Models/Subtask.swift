import Foundation
import GRDB

public struct Subtask: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var id: String
    public var taskId: String
    public var title: String
    public var done: Bool
    public var orderIndex: Int64

    public static let databaseTableName = "subtasks"

    public enum CodingKeys: String, CodingKey {
        case id
        case taskId = "task_id"
        case title
        case done
        case orderIndex = "order_index"
    }

    public init(
        id: String = UUID().uuidString.lowercased(),
        taskId: String,
        title: String,
        done: Bool = false,
        orderIndex: Int64
    ) {
        self.id = id
        self.taskId = taskId
        self.title = title
        self.done = done
        self.orderIndex = orderIndex
    }
}
