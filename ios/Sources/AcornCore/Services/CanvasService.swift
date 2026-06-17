import Foundation
import GRDB

public actor CanvasService {
    private let database: AppDatabase

    public init(database: AppDatabase) {
        self.database = database
    }

    public func list(limit: Int = 100) async throws -> [Canvas] {
        try await database.writer.read { db in
            try Canvas
                .order(Column("updated_at").desc)
                .limit(limit)
                .fetchAll(db)
        }
    }

    public func get(_ id: String) async throws -> Canvas? {
        try await database.writer.read { db in
            try Canvas.fetchOne(db, key: id)
        }
    }

    public func create(title: String? = nil) async throws -> Canvas {
        try await database.writer.write { db in
            let cleanTitle = title?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            var canvas = Canvas(
                title: cleanTitle.isEmpty ? "Untitled canvas" : cleanTitle
            )
            try canvas.insert(db)
            return canvas
        }
    }

    public func update(id: String, title: String? = nil, content: String? = nil) async throws -> Canvas {
        try await database.writer.write { db in
            guard var canvas = try Canvas.fetchOne(db, key: id) else {
                throw ProviderError.invalidResponse("Canvas \(id) not found.")
            }
            if let title = title?.trimmingCharacters(in: .whitespacesAndNewlines), !title.isEmpty {
                canvas.title = title
            }
            if let content {
                canvas.content = content
            }
            canvas.updatedAt = Date()
            try canvas.update(db)
            return canvas
        }
    }

    public func delete(_ id: String) async throws {
        _ = try await database.writer.write { db in
            try Canvas.deleteOne(db, key: id)
        }
    }
}
