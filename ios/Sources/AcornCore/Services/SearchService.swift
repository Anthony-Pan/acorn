import Foundation
import GRDB

public actor SearchService {
    private let database: AppDatabase

    public init(database: AppDatabase) {
        self.database = database
    }

    public func query(_ raw: String, limit: Int = 40) async throws -> [SearchHit] {
        guard let ftsQuery = Self.sanitize(raw) else { return [] }
        let cappedLimit = max(1, min(limit, 200))
        return try await database.writer.read { db in
            let sql = """
                SELECT s.source,
                       s.source_id,
                       s.title,
                       snippet(search_index, 3, '<<', '>>', '…', 12) AS snippet,
                       s.created_at,
                       CASE WHEN s.source = 'message' THEN
                           (SELECT conversation_id FROM messages WHERE id = s.source_id)
                       ELSE NULL END AS conversation_id
                FROM search_index s
                WHERE search_index MATCH ?
                ORDER BY rank
                LIMIT ?
            """
            let rows = try Row.fetchAll(db, sql: sql, arguments: [ftsQuery, cappedLimit])
            let formatter = ISO8601DateFormatter()
            formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            let fallback = ISO8601DateFormatter()
            return rows.compactMap { row -> SearchHit? in
                guard
                    let source: String = row["source"],
                    let sourceId: String = row["source_id"],
                    let title: String = row["title"],
                    let snippet: String = row["snippet"],
                    let createdRaw: String = row["created_at"]
                else { return nil }
                let created = formatter.date(from: createdRaw)
                    ?? fallback.date(from: createdRaw)
                    ?? Date()
                let conversationId: String? = row["conversation_id"]
                return SearchHit(
                    source: source,
                    sourceId: sourceId,
                    title: title,
                    snippet: snippet,
                    createdAt: created,
                    conversationId: conversationId
                )
            }
        }
    }

    private static func sanitize(_ raw: String) -> String? {
        let cleaned = raw.replacingOccurrences(of: "\"", with: " ")
        let words = cleaned
            .split(whereSeparator: { $0.isWhitespace })
            .filter { !$0.isEmpty }
        guard !words.isEmpty else { return nil }
        return "\"\(words.joined(separator: " "))\"*"
    }
}
