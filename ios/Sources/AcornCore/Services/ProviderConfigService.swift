import Foundation
import GRDB

public actor ProviderConfigService {
    private let database: AppDatabase
    private let keychain: Keychain

    public init(database: AppDatabase, keychain: Keychain = .acorn) {
        self.database = database
        self.keychain = keychain
    }

    public func list() async throws -> [ProviderConfig] {
        try await database.writer.read { db in
            try ProviderConfig
                .order(Column("last_used_at").desc, Column("provider_id"))
                .fetchAll(db)
        }
    }

    public func get(_ providerId: String) async throws -> ProviderConfig? {
        try await database.writer.read { db in
            try ProviderConfig.fetchOne(db, key: providerId)
        }
    }

    public func save(
        providerId: String,
        enabled: Bool,
        customEndpoint: String?,
        selectedModel: String?
    ) async throws -> ProviderConfig {
        try await database.writer.write { db in
            try db.execute(
                sql: """
                    INSERT INTO provider_configs (provider_id, enabled, custom_endpoint, selected_model)
                    VALUES (?, ?, ?, ?)
                    ON CONFLICT(provider_id) DO UPDATE SET
                        enabled = excluded.enabled,
                        custom_endpoint = excluded.custom_endpoint,
                        selected_model = excluded.selected_model
                """,
                arguments: [providerId, enabled, customEndpoint, selectedModel]
            )
            guard let config = try ProviderConfig.fetchOne(db, key: providerId) else {
                throw ProviderError.invalidResponse("Provider config not found after save.")
            }
            return config
        }
    }

    public func touch(_ providerId: String) async throws {
        _ = try await database.writer.write { db in
            try db.execute(
                sql: "UPDATE provider_configs SET last_used_at = ? WHERE provider_id = ?",
                arguments: [Date(), providerId]
            )
        }
    }

    public func saveCredentials(providerId: String, apiKey: String) throws {
        try keychain.save(account: providerId, value: apiKey)
    }

    public func deleteCredentials(providerId: String) throws {
        try keychain.delete(account: providerId)
    }

    public func hasCredentials(providerId: String) throws -> Bool {
        try keychain.has(account: providerId)
    }

    public func readCredentials(providerId: String) throws -> String? {
        try keychain.read(account: providerId)
    }
}
