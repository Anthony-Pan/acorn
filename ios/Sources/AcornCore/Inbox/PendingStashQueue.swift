import Foundation

public struct PendingStash: Codable, Sendable, Identifiable {
    public let id: String
    public let text: String
    public let createdAt: Date

    public init(text: String) {
        self.id = UUID().uuidString.lowercased()
        self.text = text
        self.createdAt = Date()
    }
}

public enum PendingStashQueue {
    public static let appGroupId = WidgetCache.appGroupId
    public static let key = "pending.stash.queue.v1"

    public static func enqueue(text: String) {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        guard let defaults = UserDefaults(suiteName: appGroupId) else { return }
        var existing = readRaw(defaults: defaults)
        existing.append(PendingStash(text: trimmed))
        if let encoded = try? JSONEncoder().encode(existing) {
            defaults.set(encoded, forKey: key)
        }
    }

    public static func drainAll() -> [PendingStash] {
        guard let defaults = UserDefaults(suiteName: appGroupId) else { return [] }
        let entries = readRaw(defaults: defaults)
        defaults.removeObject(forKey: key)
        return entries
    }

    public static func peek() -> [PendingStash] {
        guard let defaults = UserDefaults(suiteName: appGroupId) else { return [] }
        return readRaw(defaults: defaults)
    }

    private static func readRaw(defaults: UserDefaults) -> [PendingStash] {
        guard let data = defaults.data(forKey: key),
              let decoded = try? JSONDecoder().decode([PendingStash].self, from: data)
        else { return [] }
        return decoded
    }
}
