import SwiftUI
import AcornCore

public struct AcornRootContainer: View {
    @Environment(AppRouter.self) private var router
    let services: Services

    public init(services: Services) {
        self.services = services
    }

    public var body: some View {
        ZStack {
            switch router.route {
            case .input:
                InputView(speechService: services.speech)
                    .transition(.asymmetric(
                        insertion: .opacity.combined(with: .scale(scale: 1.02)),
                        removal: .opacity
                    ))
            case .stash:
                StashView()
                    .transition(.asymmetric(
                        insertion: .opacity.combined(with: .scale(scale: 1.02)),
                        removal: .opacity
                    ))
            case .settings:
                SettingsView(services: services)
                    .transition(.asymmetric(
                        insertion: .move(edge: .bottom).combined(with: .opacity),
                        removal: .opacity
                    ))
            case .history:
                HistoryView(services: services)
                    .transition(.asymmetric(
                        insertion: .move(edge: .leading).combined(with: .opacity),
                        removal: .opacity
                    ))
            case .canvas:
                CanvasView(services: services)
                    .transition(.asymmetric(
                        insertion: .move(edge: .trailing).combined(with: .opacity),
                        removal: .opacity
                    ))
            case .search:
                SearchView(services: services)
                    .transition(.asymmetric(
                        insertion: .opacity,
                        removal: .opacity
                    ))
            }
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.82), value: router.route)
    }
}
