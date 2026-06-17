import Foundation

public struct WidgetSnapshot: Codable, Sendable, Equatable {
    public struct WidgetTask: Codable, Sendable, Equatable, Identifiable {
        public let id: String
        public let title: String
        public let priority: Priority
        public let durationMinutes: Int
        public let status: TaskStatus

        public init(id: String, title: String, priority: Priority, durationMinutes: Int, status: TaskStatus) {
            self.id = id
            self.title = title
            self.priority = priority
            self.durationMinutes = durationMinutes
            self.status = status
        }
    }

    public let summary: String
    public let tasks: [WidgetTask]
    public let updatedAt: Date

    public init(summary: String, tasks: [WidgetTask], updatedAt: Date = Date()) {
        self.summary = summary
        self.tasks = tasks
        self.updatedAt = updatedAt
    }

    public static let empty = WidgetSnapshot(summary: "Nothing stashed yet.", tasks: [], updatedAt: Date(timeIntervalSince1970: 0))
}

public enum WidgetCache {
    public static let appGroupId = "group.app.acorn.shared"
    public static let snapshotKey = "today.snapshot.v1"

    public static func write(_ snapshot: WidgetSnapshot) {
        guard let defaults = UserDefaults(suiteName: appGroupId) else { return }
        guard let data = try? JSONEncoder().encode(snapshot) else { return }
        defaults.set(data, forKey: snapshotKey)
    }

    public static func read() -> WidgetSnapshot {
        guard
            let defaults = UserDefaults(suiteName: appGroupId),
            let data = defaults.data(forKey: snapshotKey),
            let snapshot = try? JSONDecoder().decode(WidgetSnapshot.self, from: data)
        else {
            return .empty
        }
        return snapshot
    }

    public static func fromTasks(_ tasks: [Task], summary: String) -> WidgetSnapshot {
        let widgetTasks = tasks.prefix(8).map { task in
            WidgetSnapshot.WidgetTask(
                id: task.id,
                title: task.title,
                priority: task.priority,
                durationMinutes: Int(task.durationMinutes),
                status: task.status
            )
        }
        return WidgetSnapshot(summary: summary, tasks: Array(widgetTasks))
    }
}
