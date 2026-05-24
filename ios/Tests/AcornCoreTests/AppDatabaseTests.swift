import Testing
import Foundation
import GRDB
@testable import AcornCore

@Suite("AppDatabase migrations + models")
struct AppDatabaseTests {
    @Test("Migrator applies all 4 migrations in order")
    func migratorAppliesAllMigrations() async throws {
        let db = try AppDatabase.makeInMemory()
        let tables = try await db.writer.read { db in
            try String.fetchAll(db, sql: """
                SELECT name FROM sqlite_master
                WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'grdb_%'
                ORDER BY name
            """)
        }
        #expect(tables.contains("sessions"))
        #expect(tables.contains("tasks"))
        #expect(tables.contains("subtasks"))
        #expect(tables.contains("provider_configs"))
        #expect(tables.contains("settings"))
        #expect(tables.contains("conversations"))
        #expect(tables.contains("messages"))
        #expect(tables.contains("speech_provider_configs"))
    }

    @Test("FTS5 search_index virtual table exists")
    func fts5SearchIndexExists() async throws {
        let db = try AppDatabase.makeInMemory()
        let exists = try await db.writer.read { db in
            try Bool.fetchOne(db, sql: """
                SELECT count(*) > 0 FROM sqlite_master
                WHERE type='table' AND name='search_index'
            """) ?? false
        }
        #expect(exists)
    }

    @Test("Round-trip Session insert + fetch")
    func sessionRoundtrip() async throws {
        let db = try AppDatabase.makeInMemory()
        let id = try await db.writer.write { db -> String in
            var session = Session(rawInput: "buy milk, call dentist", language: "en")
            try session.insert(db)
            return session.id
        }
        let fetched = try await db.writer.read { db in
            try Session.fetchOne(db, key: id)
        }
        #expect(fetched?.rawInput == "buy milk, call dentist")
        #expect(fetched?.language == "en")
        #expect(fetched?.aiSummary == nil)
    }

    @Test("Task CHECK constraint rejects invalid priority")
    func priorityCheckConstraint() async throws {
        let db = try AppDatabase.makeInMemory()
        let sessionId = try await db.writer.write { db -> String in
            var session = Session(rawInput: "x", language: "en")
            try session.insert(db)
            return session.id
        }
        await #expect(throws: DatabaseError.self) {
            try await db.writer.write { db in
                try db.execute(sql: """
                    INSERT INTO tasks (id, session_id, title, duration_minutes, priority, order_index, status, created_at)
                    VALUES (?, ?, 'test', 5, 'INVALID_PRIORITY', 0, 'pending', ?)
                """, arguments: [UUID().uuidString.lowercased(), sessionId, Date()])
            }
        }
    }

    @Test("Cascade delete: removing session removes its tasks")
    func cascadeDelete() async throws {
        let db = try AppDatabase.makeInMemory()
        let sessionId = try await db.writer.write { db -> String in
            var session = Session(rawInput: "x", language: "en")
            try session.insert(db)
            var task = Task(
                sessionId: session.id,
                title: "fix bug",
                durationMinutes: 30,
                priority: .high,
                orderIndex: 0
            )
            try task.insert(db)
            return session.id
        }
        let beforeDelete = try await db.writer.read { db in
            try Task.fetchCount(db)
        }
        #expect(beforeDelete == 1)
        _ = try await db.writer.write { db in
            try Session.deleteOne(db, key: sessionId)
        }
        let afterDelete = try await db.writer.read { db in
            try Task.fetchCount(db)
        }
        #expect(afterDelete == 0)
    }

    @Test("Priority enum round-trips via DatabaseValueConvertible")
    func priorityRoundtrip() async throws {
        let db = try AppDatabase.makeInMemory()
        try await db.writer.write { db in
            var session = Session(rawInput: "x", language: "en")
            try session.insert(db)
            for (idx, p) in Priority.allCases.enumerated() {
                var task = Task(
                    sessionId: session.id,
                    title: "task \(p.rawValue)",
                    durationMinutes: 1,
                    priority: p,
                    orderIndex: Int64(idx)
                )
                try task.insert(db)
            }
        }
        let priorities = try await db.writer.read { db in
            try Task.order(Column("order_index")).fetchAll(db).map(\.priority)
        }
        #expect(priorities == [.high, .medium, .low])
    }

    @Test("TaskStatus enum round-trips (in_progress maps correctly)")
    func taskStatusRoundtrip() async throws {
        let db = try AppDatabase.makeInMemory()
        try await db.writer.write { db in
            var session = Session(rawInput: "x", language: "en")
            try session.insert(db)
            for (idx, s) in TaskStatus.allCases.enumerated() {
                var task = Task(
                    sessionId: session.id,
                    title: "task \(s.rawValue)",
                    durationMinutes: 1,
                    priority: .medium,
                    orderIndex: Int64(idx),
                    status: s
                )
                try task.insert(db)
            }
        }
        let statuses = try await db.writer.read { db in
            try Task.order(Column("order_index")).fetchAll(db).map(\.status)
        }
        #expect(statuses == [.pending, .inProgress, .completed, .skipped])
    }

    @Test("Settings key/value upsert via SettingRow")
    func settingsUpsert() async throws {
        let db = try AppDatabase.makeInMemory()
        try await db.writer.write { db in
            try db.execute(
                sql: "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                arguments: ["theme", "dark"]
            )
        }
        let value = try await db.writer.read { db in
            try SettingRow.fetchOne(db, key: "theme")
        }
        #expect(value?.value == "dark")
    }
}

@Suite("Provider catalog")
struct ProviderCatalogTests {
    @Test("Catalog has 16 providers")
    func catalogHasSixteenProviders() {
        #expect(ProviderCatalog.all.count == 16)
    }

    @Test("Anthropic is the default and featured")
    func anthropicIsDefault() throws {
        let anthropic = try #require(ProviderCatalog.get("anthropic"))
        #expect(anthropic.featured == true)
        #expect(anthropic.apiFormat == .anthropic)
        #expect(ProviderCatalog.defaultId == "anthropic")
    }

    @Test("Ollama and CLI providers are marked unsupported on iOS")
    func ollamaUnsupportedOnIos() throws {
        let ollama = try #require(ProviderCatalog.get("ollama"))
        #expect(ollama.status == .platformUnsupported)
    }

    @Test("FoundationModels exists as a local provider")
    func foundationModelsExists() throws {
        let fm = try #require(ProviderCatalog.get("foundation_models"))
        #expect(fm.category == .local)
        #expect(fm.requiresApiKey == false)
    }

    @Test("Each provider id is unique")
    func providerIdsAreUnique() {
        let ids = ProviderCatalog.all.map(\.id)
        #expect(Set(ids).count == ids.count)
    }
}
