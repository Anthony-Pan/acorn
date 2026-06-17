import Foundation

public enum MigrationLoader {
    public struct File: Sendable, Equatable {
        public let identifier: String
        public let resourceName: String

        public init(identifier: String, resourceName: String) {
            self.identifier = identifier
            self.resourceName = resourceName
        }
    }

    public static let allMigrations: [File] = [
        File(identifier: "0001_initial", resourceName: "0001_initial"),
        File(identifier: "0002_chat", resourceName: "0002_chat"),
        File(identifier: "0003_search", resourceName: "0003_search"),
        File(identifier: "0004_speech_providers", resourceName: "0004_speech_providers"),
        File(identifier: "0005_activity_log", resourceName: "0005_activity_log"),
        File(identifier: "0006_canvases", resourceName: "0006_canvases"),
    ]

    public enum LoadError: Error, CustomStringConvertible {
        case missingResource(String)

        public var description: String {
            switch self {
            case .missingResource(let name):
                "Migration resource \(name).sql is missing from Bundle.module"
            }
        }
    }

    public static func sql(for file: File) throws -> String {
        guard let url = Bundle.module.url(forResource: file.resourceName, withExtension: "sql") else {
            throw LoadError.missingResource(file.resourceName)
        }
        return try String(contentsOf: url, encoding: .utf8)
    }
}
