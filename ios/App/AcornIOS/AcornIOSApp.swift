import SwiftUI
import AcornApp
import AcornCore

@main
struct AcornIOSApp: App {
    init() {
        bootstrap()
    }

    var body: some Scene {
        WindowGroup {
            RootView()
        }
    }
}
