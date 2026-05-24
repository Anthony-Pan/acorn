import Foundation
import GRDB

public enum Priority: String, Codable, Sendable, CaseIterable, DatabaseValueConvertible {
    case high
    case medium
    case low
}
