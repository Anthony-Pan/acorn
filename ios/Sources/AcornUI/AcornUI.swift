import SwiftUI
import AcornCore

public struct AcornRootContainer: View {
    @Environment(AppRouter.self) private var router
    let speechService: SpeechService

    public init(speechService: SpeechService) {
        self.speechService = speechService
    }

    public var body: some View {
        ZStack {
            switch router.route {
            case .input:
                InputView(speechService: speechService)
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
                SettingsView()
                    .transition(.asymmetric(
                        insertion: .move(edge: .bottom).combined(with: .opacity),
                        removal: .opacity
                    ))
            }
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.82), value: router.route)
    }
}
