import SwiftUI
import AcornCore
import AcornUI

public struct RootView: View {
    private let services: Services

    @State private var router: AppRouter
    @State private var settings: SettingsStore
    @State private var providers: ProvidersStore
    @State private var session: SessionStore

    public init(services: Services) {
        self.services = services
        let router = AppRouter()
        let settings = SettingsStore(service: services.settings)
        let providers = ProvidersStore(service: services.providerConfigs)
        let session = SessionStore(services: services)
        _router = State(initialValue: router)
        _settings = State(initialValue: settings)
        _providers = State(initialValue: providers)
        _session = State(initialValue: session)
    }

    public var body: some View {
        AcornRootContainer(services: services)
            .environment(router)
            .environment(settings)
            .environment(providers)
            .environment(session)
            .task {
                await settings.hydrate()
                await providers.hydrate()
                await session.hydrateLatestSession()
                await session.drainPendingShareExtensionInbox(
                    activeProviderId: settings.activeProviderId,
                    language: settings.language.isEmpty ? "en" : settings.language
                )
                let isActive: Bool
                if case .idle = session.stashState { isActive = false } else { isActive = true }
                if !session.tasks.isEmpty || (session.currentSession != nil && isActive) {
                    router.navigate(.stash)
                }
            }
    }
}
