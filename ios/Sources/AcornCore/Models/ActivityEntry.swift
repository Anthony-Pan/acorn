import Foundation
import GRDB

public struct ActivityEntry: Codable, Sendable, Identifiable, FetchableRecord, MutablePersistableRecord {
    public var id: String
    public var kind: String
    public var content: String
    public var createdAt: Date

    public static let databaseTableName = "activity_log"

    public enum CodingKeys: String, CodingKey {
        case id
        case kind
        case content
        case createdAt = "created_at"
    }

    public init(
        id: String = UUID().uuidString.lowercased(),
        kind: String,
        content: String,
        createdAt: Date = Date()
    ) {
        self.id = id
        self.kind = kind
        self.content = content
        self.createdAt = createdAt
    }
}

public enum ActivityKind: String, Sendable {
    case stash
    case taskComplete = "task_complete"
    case taskSkip = "task_skip"
    case chat
    case search
    case canvasEdit = "canvas_edit"
    case voiceTranscribe = "voice_transcribe"
}
