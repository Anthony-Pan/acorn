import Foundation
import Observation

@MainActor
@Observable
public final class AppRouter {
    public enum Route: Hashable, Sendable {
        case input
        case stash
        case settings
        case history
    }

    public var route: Route = .input
    public var activeSessionId: String?

    public init() {}

    public func navigate(_ next: Route) {
        route = next
    }
}
