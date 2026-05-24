import Foundation
import GRDB

public actor TaskService {
    private let database: AppDatabase

    public init(database: AppDatabase) {
        self.database = database
    }

    public func insert(
        sessionId: String,
        title: String,
        description: String?,
        durationMinutes: Int64,
        priority: Priority,
        orderIndex: Int64,
        subtasks: [DecomposedSubtask] = []
    ) async throws -> Task {
        try await database.writer.write { db in
            var task = Task(
                sessionId: sessionId,
                title: title,
                description: description,
                durationMinutes: durationMinutes,
                priority: priority,
                orderIndex: orderIndex
            )
            try task.insert(db)
            for (idx, st) in subtasks.enumerated() {
                var subtask = Subtask(
                    taskId: task.id,
                    title: st.title,
                    done: st.done,
                    orderIndex: Int64(idx)
                )
                try subtask.insert(db)
            }
            return task
        }
    }

    public func updateStatus(taskId: String, status: TaskStatus) async throws -> Task {
        try await database.writer.write { db in
            switch status {
            case .inProgress:
                try db.execute(
                    sql: """
                        UPDATE tasks SET
                            status = ?,
                            started_at = COALESCE(started_at, ?),
                            completed_at = NULL
                        WHERE id = ?
                    """,
                    arguments: [status.rawValue, Date(), taskId]
                )
            case .completed, .skipped:
                try db.execute(
                    sql: "UPDATE tasks SET status = ?, completed_at = ? WHERE id = ?",
                    arguments: [status.rawValue, Date(), taskId]
                )
            case .pending:
                try db.execute(
                    sql: "UPDATE tasks SET status = ?, started_at = NULL, completed_at = NULL WHERE id = ?",
                    arguments: [status.rawValue, taskId]
                )
            }
            guard let task = try Task.fetchOne(db, key: taskId) else {
                throw ProviderError.invalidResponse("Task \(taskId) not found.")
            }
            return task
        }
    }

    public func subtasks(for taskId: String) async throws -> [Subtask] {
        try await database.writer.read { db in
            try Subtask
                .filter(Column("task_id") == taskId)
                .order(Column("order_index"))
                .fetchAll(db)
        }
    }

    public func toggleSubtask(_ subtaskId: String) async throws -> Subtask {
        try await database.writer.write { db in
            try db.execute(
                sql: "UPDATE subtasks SET done = 1 - done WHERE id = ?",
                arguments: [subtaskId]
            )
            guard let subtask = try Subtask.fetchOne(db, key: subtaskId) else {
                throw ProviderError.invalidResponse("Subtask \(subtaskId) not found.")
            }
            return subtask
        }
    }
}
