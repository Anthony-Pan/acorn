import Foundation

#if canImport(ActivityKit) && os(iOS)
@preconcurrency import ActivityKit

@available(iOS 16.2, *)
public struct TaskActivityAttributes: ActivityAttributes {
    public struct ContentState: Codable, Hashable, Sendable {
        public var elapsedSeconds: Int
        public var status: TaskStatus

        public init(elapsedSeconds: Int, status: TaskStatus) {
            self.elapsedSeconds = elapsedSeconds
            self.status = status
        }
    }

    public let taskId: String
    public let title: String
    public let priority: Priority
    public let durationMinutes: Int
    public let startedAt: Date

    public init(
        taskId: String,
        title: String,
        priority: Priority,
        durationMinutes: Int,
        startedAt: Date
    ) {
        self.taskId = taskId
        self.title = title
        self.priority = priority
        self.durationMinutes = durationMinutes
        self.startedAt = startedAt
    }
}

@available(iOS 16.2, *)
@MainActor
public enum TaskActivityController {
    public static func start(for task: Task) async {
        guard task.status == .inProgress else { return }
        guard ActivityAuthorizationInfo().areActivitiesEnabled else { return }
        if findActivity(taskId: task.id) != nil { return }
        let attrs = TaskActivityAttributes(
            taskId: task.id,
            title: task.title,
            priority: task.priority,
            durationMinutes: Int(task.durationMinutes),
            startedAt: task.startedAt ?? Date()
        )
        let initial = TaskActivityAttributes.ContentState(
            elapsedSeconds: 0,
            status: task.status
        )
        do {
            _ = try Activity<TaskActivityAttributes>.request(
                attributes: attrs,
                content: .init(state: initial, staleDate: nil)
            )
        } catch {
            // Live Activities aren't always available (e.g. disabled in
            // Settings, low power mode); failing silently is fine — the
            // app + widget still convey the same information.
        }
    }

    public static func update(for task: Task) async {
        guard let activity = findActivity(taskId: task.id) else { return }
        let elapsed = Int(Date().timeIntervalSince(task.startedAt ?? activity.attributes.startedAt))
        let state = TaskActivityAttributes.ContentState(
            elapsedSeconds: max(0, elapsed),
            status: task.status
        )
        await activity.update(.init(state: state, staleDate: nil))
    }

    public static func end(for task: Task) async {
        guard let activity = findActivity(taskId: task.id) else { return }
        let finalState = TaskActivityAttributes.ContentState(
            elapsedSeconds: Int(Date().timeIntervalSince(activity.attributes.startedAt)),
            status: task.status
        )
        await activity.end(.init(state: finalState, staleDate: nil), dismissalPolicy: .immediate)
    }

    private static func findActivity(taskId: String) -> Activity<TaskActivityAttributes>? {
        Activity<TaskActivityAttributes>.activities.first { $0.attributes.taskId == taskId }
    }
}

#endif
