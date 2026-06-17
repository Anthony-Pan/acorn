import SwiftUI
import AcornCore

public struct PrivacyPanel: View {
    let services: Services
    @State private var entries: [ActivityEntry] = []
    @State private var loading = true
    @State private var showCleared = false

    public init(services: Services) {
        self.services = services
    }

    public var body: some View {
        List {
            Section {
                Text("Acorn logs your actions locally to power morning brief, weekly recap, and on-device search. Nothing here leaves your phone.")
                    .font(.acornCaption)
                    .foregroundStyle(.secondary)
            }

            Section("Recent activity (\(entries.count))") {
                if loading {
                    ProgressView()
                } else if entries.isEmpty {
                    Text("No activity logged yet.")
                        .font(.acornCaption)
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(entries.prefix(50)) { entry in
                        VStack(alignment: .leading, spacing: 2) {
                            Text(entry.kind.replacingOccurrences(of: "_", with: " ").capitalized)
                                .font(.acornCaption.weight(.semibold))
                                .foregroundStyle(Color.acornAccent)
                            Text(entry.content)
                                .font(.acornBody)
                                .lineLimit(3)
                            Text(entry.createdAt, style: .relative)
                                .font(.acornCaption)
                                .foregroundStyle(.secondary)
                        }
                        .padding(.vertical, 2)
                    }
                }
            }

            Section {
                Button(role: .destructive) {
                    _Concurrency.Task { await clearAll() }
                } label: {
                    Label("Clear activity log", systemImage: "trash")
                }
                .disabled(entries.isEmpty)
            }

            if showCleared {
                Text("Cleared.")
                    .font(.acornCaption)
                    .foregroundStyle(Color.mossGreen)
            }
        }
        .navigationTitle("Privacy")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task {
            await load()
        }
    }

    private func load() async {
        loading = true
        defer { loading = false }
        entries = (try? await services.activity.listRecent(limit: 200)) ?? []
    }

    private func clearAll() async {
        try? await services.activity.clear()
        entries = []
        showCleared = true
        try? await _Concurrency.Task.sleep(for: .seconds(1.5))
        showCleared = false
    }
}
