import Foundation
import GRDB

public actor ActivityService {
    private let database: AppDatabase

    public init(database: AppDatabase) {
        self.database = database
    }

    private static let sensitivePatterns: [String] = [
        "password", "passwd", "credential", "secret", "api_key", "apikey",
        "token=", ".env", "salary",
        "薪资", "合同", "密码", "信用卡",
    ]

    private static func isSensitive(_ content: String) -> Bool {
        let lower = content.lowercased()
        return sensitivePatterns.contains { lower.contains($0) }
    }

    @discardableResult
    public func record(kind: ActivityKind, content: String) async throws -> ActivityEntry? {
        let trimmed = content.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, !Self.isSensitive(trimmed) else { return nil }
        let truncated = trimmed.count > 500
            ? String(trimmed.prefix(500)) + "…"
            : trimmed
        return try await database.writer.write { db in
            var entry = ActivityEntry(kind: kind.rawValue, content: truncated)
            try entry.insert(db)
            return entry
        }
    }

    public func listRecent(limit: Int = 100) async throws -> [ActivityEntry] {
        try await database.writer.read { db in
            try ActivityEntry
                .order(Column("created_at").desc)
                .limit(limit)
                .fetchAll(db)
        }
    }

    public func clear() async throws {
        _ = try await database.writer.write { db in
            try db.execute(sql: "DELETE FROM activity_log")
        }
    }

    public func count() async throws -> Int {
        try await database.writer.read { db in
            try ActivityEntry.fetchCount(db)
        }
    }
}
