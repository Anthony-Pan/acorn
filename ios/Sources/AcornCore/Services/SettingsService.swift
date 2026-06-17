import Foundation
import GRDB

public actor SettingsService {
    private let database: AppDatabase

    public init(database: AppDatabase) {
        self.database = database
    }

    public func get(_ key: SettingKey) async throws -> String? {
        try await get(key.rawValue)
    }

    public func get(_ key: String) async throws -> String? {
        try await database.writer.read { db in
            try SettingRow.fetchOne(db, key: key)?.value
        }
    }

    public func set(_ key: SettingKey, value: String) async throws {
        try await set(key.rawValue, value: value)
    }

    public func set(_ key: String, value: String) async throws {
        _ = try await database.writer.write { db in
            try db.execute(
                sql: "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                arguments: [key, value]
            )
        }
    }

    public func remove(_ key: SettingKey) async throws {
        try await remove(key.rawValue)
    }

    public func remove(_ key: String) async throws {
        _ = try await database.writer.write { db in
            try db.execute(sql: "DELETE FROM settings WHERE key = ?", arguments: [key])
        }
    }
}
