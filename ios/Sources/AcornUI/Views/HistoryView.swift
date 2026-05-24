import SwiftUI
import AcornCore

public struct HistoryView: View {
    @Environment(AppRouter.self) private var router
    @Environment(SessionStore.self) private var session
    let services: Services

    @State private var sessions: [Session] = []
    @State private var loading = true

    public init(services: Services) {
        self.services = services
    }

    public var body: some View {
        NavigationStack {
            Group {
                if loading {
                    ProgressView().controlSize(.large).tint(Color.acornAccent)
                } else if sessions.isEmpty {
                    emptyState
                } else {
                    list
                }
            }
            .navigationTitle("History")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { router.navigate(.input) }
                        .font(.acornButton)
                }
            }
            .background(Color.warmCream.opacity(0.4).ignoresSafeArea())
        }
        .task {
            await load()
        }
    }

    private var list: some View {
        List {
            ForEach(groupedByDay, id: \.0) { day, items in
                Section(day) {
                    ForEach(items) { item in
                        Button {
                            openSession(item)
                        } label: {
                            SessionRow(session: item)
                        }
                        .buttonStyle(.plain)
                    }
                }
            }
        }
        #if os(iOS)
        .listStyle(.insetGrouped)
        #endif
        .scrollContentBackground(.hidden)
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Text("🌰").font(.system(size: 56))
            Text("No past stashes yet").font(.acornHeader).foregroundStyle(.secondary)
            Button("Stash something") { router.navigate(.input) }
                .font(.acornButton)
                .foregroundStyle(Color.acornAccent)
        }
    }

    private var groupedByDay: [(String, [Session])] {
        let groups = Dictionary(grouping: sessions) { session in
            Self.dayFormatter.string(from: session.createdAt)
        }
        return groups
            .sorted { lhs, rhs in
                guard
                    let l = Self.dayFormatter.date(from: lhs.key),
                    let r = Self.dayFormatter.date(from: rhs.key)
                else { return lhs.key > rhs.key }
                return l > r
            }
            .map { ($0.key, $0.value.sorted(by: { $0.createdAt > $1.createdAt })) }
    }

    private static let dayFormatter: DateFormatter = {
        let df = DateFormatter()
        df.dateStyle = .full
        df.timeStyle = .none
        return df
    }()

    private func load() async {
        loading = true
        defer { loading = false }
        if let recent = try? await services.sessions.listRecent(limit: 200) {
            sessions = recent
        }
    }

    private func openSession(_ s: Session) {
        _Concurrency.Task { @MainActor in
            session.reset()
            if let full = try? await services.sessions.get(s.id),
               let tasks = try? await services.sessions.tasks(for: full.id) {
                session.currentSession = full
                session.tasks = tasks
                session.summary = full.aiSummary ?? ""
                session.stashState = .done
            }
            router.navigate(.stash)
        }
    }
}

struct SessionRow: View {
    let session: Session

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(timeOfDay)
                .font(.acornCaption.weight(.semibold))
                .foregroundStyle(Color.acornAccent)
            Text(session.aiSummary ?? session.rawInput)
                .font(.acornBody)
                .lineLimit(2)
                .foregroundStyle(.primary)
            HStack(spacing: 6) {
                if let provider = session.providerId {
                    Label(provider, systemImage: "sparkles")
                        .font(.acornCaption)
                        .foregroundStyle(.secondary)
                }
                if session.completedAt != nil {
                    Label("Stashed", systemImage: "checkmark.circle")
                        .font(.acornCaption)
                        .foregroundStyle(Color.mossGreen)
                } else {
                    Label("In progress", systemImage: "circle.dotted")
                        .font(.acornCaption)
                        .foregroundStyle(Color.autumnAmber)
                }
            }
        }
        .padding(.vertical, 4)
    }

    private var timeOfDay: String {
        Self.timeFormatter.string(from: session.createdAt).uppercased()
    }

    private static let timeFormatter: DateFormatter = {
        let df = DateFormatter()
        df.dateStyle = .none
        df.timeStyle = .short
        return df
    }()
}
