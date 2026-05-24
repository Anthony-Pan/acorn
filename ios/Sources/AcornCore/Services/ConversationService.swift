import Foundation
import GRDB

public actor ConversationService {
    private let database: AppDatabase

    public init(database: AppDatabase) {
        self.database = database
    }

    public func create(title: String? = nil) async throws -> Conversation {
        try await database.writer.write { db in
            let cleanTitle = title?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            var convo = Conversation(title: cleanTitle.isEmpty ? "Untitled" : cleanTitle)
            try convo.insert(db)
            return convo
        }
    }

    public func list(limit: Int = 100) async throws -> [Conversation] {
        try await database.writer.read { db in
            try Conversation
                .order(Column("last_message_at").desc)
                .limit(limit)
                .fetchAll(db)
        }
    }

    public func get(_ id: String) async throws -> Conversation? {
        try await database.writer.read { db in
            try Conversation.fetchOne(db, key: id)
        }
    }

    public func messages(for conversationId: String) async throws -> [Message] {
        try await database.writer.read { db in
            try Message
                .filter(Column("conversation_id") == conversationId)
                .order(Column("created_at"))
                .fetchAll(db)
        }
    }

    public func delete(_ id: String) async throws {
        _ = try await database.writer.write { db in
            try Conversation.deleteOne(db, key: id)
        }
    }

    public func rename(_ id: String, title: String) async throws -> Conversation {
        try await database.writer.write { db in
            guard var convo = try Conversation.fetchOne(db, key: id) else {
                throw ProviderError.invalidResponse("Conversation \(id) not found.")
            }
            let cleaned = title.trimmingCharacters(in: .whitespacesAndNewlines)
            convo.title = cleaned.isEmpty ? "Untitled" : cleaned
            try convo.update(db)
            return convo
        }
    }

    public func setProvider(_ id: String, providerId: String, model: String?) async throws -> Conversation {
        try await database.writer.write { db in
            guard var convo = try Conversation.fetchOne(db, key: id) else {
                throw ProviderError.invalidResponse("Conversation \(id) not found.")
            }
            convo.providerId = providerId
            convo.model = model
            try convo.update(db)
            return convo
        }
    }

    public func appendMessage(
        conversationId: String,
        role: MessageRole,
        content: String,
        toolCalls: String? = nil,
        toolCallId: String? = nil
    ) async throws -> Message {
        try await database.writer.write { db in
            var msg = Message(
                conversationId: conversationId,
                role: role,
                content: content,
                toolCalls: toolCalls,
                toolCallId: toolCallId
            )
            try msg.insert(db)
            try db.execute(
                sql: "UPDATE conversations SET last_message_at = ? WHERE id = ?",
                arguments: [Date(), conversationId]
            )
            return msg
        }
    }
}
