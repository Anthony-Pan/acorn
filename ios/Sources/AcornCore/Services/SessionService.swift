import Foundation
import GRDB

public actor SessionService {
    private let database: AppDatabase

    public init(database: AppDatabase) {
        self.database = database
    }

    public func create(rawInput: String, language: String) async throws -> Session {
        try await database.writer.write { db in
            var session = Session(rawInput: rawInput, language: language)
            try session.insert(db)
            return session
        }
    }

    public func finalize(
        sessionId: String,
        aiSummary: String,
        providerId: String,
        model: String
    ) async throws -> Session {
        try await database.writer.write { db in
            try db.execute(
                sql: """
                    UPDATE sessions SET
                        ai_summary = ?,
                        provider_id = ?,
                        model = ?,
                        completed_at = ?
                    WHERE id = ?
                """,
                arguments: [aiSummary, providerId, model, Date(), sessionId]
            )
            guard let session = try Session.fetchOne(db, key: sessionId) else {
                throw ProviderError.invalidResponse("Session \(sessionId) vanished after update.")
            }
            return session
        }
    }

    public func get(_ id: String) async throws -> Session? {
        try await database.writer.read { db in
            try Session.fetchOne(db, key: id)
        }
    }

    public func listToday() async throws -> [Session] {
        try await database.writer.read { db in
            try Session
                .filter(sql: "date(created_at) = date('now', 'localtime')")
                .order(Column("created_at").desc)
                .fetchAll(db)
        }
    }

    public func listRecent(limit: Int = 50) async throws -> [Session] {
        try await database.writer.read { db in
            try Session
                .order(Column("created_at").desc)
                .limit(limit)
                .fetchAll(db)
        }
    }

    public func tasks(for sessionId: String) async throws -> [Task] {
        try await database.writer.read { db in
            try Task
                .filter(Column("session_id") == sessionId)
                .order(Column("order_index"))
                .fetchAll(db)
        }
    }
}
