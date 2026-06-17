import Foundation

public struct ChatTurn: Codable, Sendable {
    public let role: String
    public let content: String

    public init(role: String, content: String) {
        self.role = role
        self.content = content
    }
}

public enum ChatRequestError: Error, LocalizedError, Sendable {
    case providerMismatch(locked: String, attempted: String)
    case empty

    public var errorDescription: String? {
        switch self {
        case .providerMismatch(let locked, let attempted):
            return "This conversation is locked to '\(locked)'. To use '\(attempted)', start a new conversation."
        case .empty:
            return "Empty message."
        }
    }
}
