import Foundation
import GRDB

public struct SettingRow: Codable, Sendable, FetchableRecord, MutablePersistableRecord {
    public var key: String
    public var value: String

    public static let databaseTableName = "settings"

    public init(key: String, value: String) {
        self.key = key
        self.value = value
    }
}

public enum SettingKey: String, Sendable, CaseIterable {
    case activeProvider = "active_provider"
    case activeSpeechProvider = "active_speech_provider"
    case theme
    case language
    case summonShortcut = "summon_shortcut"
}
