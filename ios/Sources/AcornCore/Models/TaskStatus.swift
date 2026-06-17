import Foundation
import GRDB

public enum TaskStatus: String, Codable, Sendable, CaseIterable, DatabaseValueConvertible {
    case pending
    case inProgress = "in_progress"
    case completed
    case skipped
}
