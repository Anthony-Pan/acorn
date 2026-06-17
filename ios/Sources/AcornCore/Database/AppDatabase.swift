import Foundation
import GRDB
import OSLog

private let logger = Logger(subsystem: "app.acorn.ios", category: "AppDatabase")

public actor AppDatabase {
    public let writer: any DatabaseWriter

    public init(writer: any DatabaseWriter) throws {
        self.writer = writer
        try Self.migrator.migrate(writer)
    }

    public static func makeOnDisk(at url: URL) throws -> AppDatabase {
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        var config = Configuration()
        config.maximumReaderCount = 5
        config.prepareDatabase { db in
            try db.execute(sql: "PRAGMA foreign_keys = ON;")
        }
        let pool = try DatabasePool(path: url.path, configuration: config)
        return try AppDatabase(writer: pool)
    }

    public static func makeInMemory() throws -> AppDatabase {
        var config = Configuration()
        config.prepareDatabase { db in
            try db.execute(sql: "PRAGMA foreign_keys = ON;")
        }
        let queue = try DatabaseQueue(configuration: config)
        return try AppDatabase(writer: queue)
    }

    public static let defaultDatabaseURL: URL = {
        let appSupport = FileManager.default.urls(
            for: .applicationSupportDirectory, in: .userDomainMask
        ).first!
        return appSupport
            .appendingPathComponent("app.acorn.ios", isDirectory: true)
            .appendingPathComponent("acorn.db")
    }()

    static var migrator: DatabaseMigrator {
        var migrator = DatabaseMigrator()

        for migrationFile in MigrationLoader.allMigrations {
            migrator.registerMigration(migrationFile.identifier) { db in
                logger.info("Running migration \(migrationFile.identifier, privacy: .public)")
                let sql = try MigrationLoader.sql(for: migrationFile)
                try db.execute(sql: sql)
            }
        }

        return migrator
    }
}
