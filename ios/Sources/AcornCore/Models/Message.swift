import Foundation
import GRDB

public struct Message: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var id: String
    public var conversationId: String
    public var role: MessageRole
    public var content: String
    public var toolCalls: String?
    public var toolCallId: String?
    public var createdAt: Date

    public static let databaseTableName = "messages"

    public enum CodingKeys: String, CodingKey {
        case id
        case conversationId = "conversation_id"
        case role
        case content
        case toolCalls = "tool_calls"
        case toolCallId = "tool_call_id"
        case createdAt = "created_at"
    }

    public init(
        id: String = UUID().uuidString.lowercased(),
        conversationId: String,
        role: MessageRole,
        content: String,
        toolCalls: String? = nil,
        toolCallId: String? = nil,
        createdAt: Date = Date()
    ) {
        self.id = id
        self.conversationId = conversationId
        self.role = role
        self.content = content
        self.toolCalls = toolCalls
        self.toolCallId = toolCallId
        self.createdAt = createdAt
    }
}
