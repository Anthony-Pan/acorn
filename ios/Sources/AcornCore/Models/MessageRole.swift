import Foundation
import GRDB

public enum MessageRole: String, Codable, Sendable, CaseIterable, DatabaseValueConvertible {
    case user
    case assistant
    case tool
    case system
}
