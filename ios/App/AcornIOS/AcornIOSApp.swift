import SwiftUI
import AcornApp
import AcornCore

@main
struct AcornIOSApp: App {
    @State private var bootstrap = AppBootstrap()

    var body: some Scene {
        WindowGroup {
            Group {
                if let services = bootstrap.services {
                    RootView(services: services)
                } else if let error = bootstrap.error {
                    BootstrapErrorView(error: error)
                } else {
                    BootstrapLoadingView()
                }
            }
            .task {
                await bootstrap.loadIfNeeded()
            }
        }
    }
}

@Observable
@MainActor
final class AppBootstrap {
    var services: Services?
    var error: String?

    func loadIfNeeded() async {
        guard services == nil, error == nil else { return }
        do {
            services = try await AcornBootstrap.makeServices()
        } catch {
            self.error = error.localizedDescription
        }
    }
}

struct BootstrapLoadingView: View {
    var body: some View {
        VStack(spacing: 16) {
            ProgressView().controlSize(.large)
            Text("Cracking the acorn…")
                .foregroundStyle(.secondary)
        }
    }
}

struct BootstrapErrorView: View {
    let error: String
    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.largeTitle)
                .foregroundStyle(.red)
            Text("Couldn't start Acorn")
                .font(.title2.weight(.semibold))
            Text(error)
                .font(.callout)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)
        }
    }
}
