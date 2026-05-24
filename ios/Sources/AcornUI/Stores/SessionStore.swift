import Foundation
import Observation
import AcornCore

@MainActor
@Observable
public final class SessionStore {
    public enum StashState: Sendable {
        case idle
        case awaitingFirstTask
        case streaming
        case done
        case failed(String)
    }

    public var currentSession: Session?
    public var tasks: [Task] = []
    public var summary: String = ""
    public var stashState: StashState = .idle

    private let services: Services

    public init(services: Services) {
        self.services = services
    }

    public func reset() {
        currentSession = nil
        tasks = []
        summary = ""
        stashState = .idle
    }

    public func stash(rawInput: String, language: String, providerId: String) async {
        reset()
        stashState = .awaitingFirstTask
        let stream = await services.ai.stash(
            rawInput: rawInput,
            language: language,
            providerId: providerId
        )
        do {
            for try await event in stream {
                switch event {
                case .sessionCreated(let session):
                    currentSession = session
                case .progress:
                    if case .awaitingFirstTask = stashState {
                        stashState = .streaming
                    }
                case .task(let task):
                    tasks.append(task)
                    stashState = .streaming
                case .summary(let s):
                    summary = s
                case .done:
                    stashState = .done
                }
            }
        } catch {
            stashState = .failed(error.localizedDescription)
        }
    }

    public func hydrateLatestSession() async {
        do {
            let recent = try await services.sessions.listRecent(limit: 1)
            if let session = recent.first {
                let tasks = try await services.sessions.tasks(for: session.id)
                self.currentSession = session
                self.tasks = tasks
                self.summary = session.aiSummary ?? ""
                self.stashState = .done
            }
        } catch {
            // Silent; UI will fall back to empty state.
        }
    }

    public func toggleStatus(taskId: String) async {
        guard let idx = tasks.firstIndex(where: { $0.id == taskId }) else { return }
        let task = tasks[idx]
        let nextStatus: TaskStatus = switch task.status {
        case .pending: .inProgress
        case .inProgress: .completed
        case .completed: .pending
        case .skipped: .pending
        }
        if let updated = try? await services.tasks.updateStatus(taskId: taskId, status: nextStatus) {
            tasks[idx] = updated
        }
    }

    public func skip(taskId: String) async {
        if let updated = try? await services.tasks.updateStatus(taskId: taskId, status: .skipped) {
            if let idx = tasks.firstIndex(where: { $0.id == taskId }) {
                tasks[idx] = updated
            }
        }
    }
}
