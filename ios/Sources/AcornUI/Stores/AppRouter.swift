import Foundation
import Observation

@MainActor
@Observable
public final class AppRouter {
    public enum Route: Hashable, Sendable {
        case input
        case stash
        case settings
    }

    public var route: Route = .input
    public var activeSessionId: String?

    public init() {}

    public func navigate(_ next: Route) {
        route = next
    }
}
