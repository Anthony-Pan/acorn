import Foundation

public struct DecomposeRequest: Codable, Sendable {
    public var rawInput: String
    public var language: String
    public var userContext: String?

    public init(rawInput: String, language: String, userContext: String? = nil) {
        self.rawInput = rawInput
        self.language = language
        self.userContext = userContext
    }

    public enum CodingKeys: String, CodingKey {
        case rawInput
        case language
        case userContext
    }
}

public struct DecomposedSubtask: Codable, Sendable, Identifiable {
    public var id: String
    public var title: String
    public var done: Bool

    public init(id: String = UUID().uuidString.lowercased(), title: String, done: Bool = false) {
        self.id = id
        self.title = title
        self.done = done
    }
}

public struct DecomposedTask: Codable, Sendable, Identifiable {
    public var id: String
    public var title: String
    public var description: String?
    public var durationMinutes: Int
    public var priority: Priority
    public var order: Int
    public var subtasks: [DecomposedSubtask]

    public init(
        id: String = UUID().uuidString.lowercased(),
        title: String,
        description: String? = nil,
        durationMinutes: Int,
        priority: Priority,
        order: Int,
        subtasks: [DecomposedSubtask] = []
    ) {
        self.id = id
        self.title = title
        self.description = description
        self.durationMinutes = max(1, min(durationMinutes, 480))
        self.priority = priority
        self.order = order
        self.subtasks = subtasks
    }
}

public struct DecomposeResponse: Codable, Sendable {
    public var tasks: [DecomposedTask]
    public var summary: String

    public init(tasks: [DecomposedTask], summary: String) {
        self.tasks = tasks
        self.summary = summary
    }
}

public enum DecomposeEvent: Sendable {
    case progress(receivedChars: Int)
    case task(DecomposedTask)
    case summary(String)
    case done
}
