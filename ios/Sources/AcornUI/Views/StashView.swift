import SwiftUI
import AcornCore

public struct StashView: View {
    @Environment(SessionStore.self) private var session
    @Environment(AppRouter.self) private var router

    public init() {}

    public var body: some View {
        NavigationStack {
            ScrollView {
                LazyVStack(spacing: 16) {
                    header
                    if session.tasks.isEmpty {
                        emptyState
                    } else {
                        ForEach(session.tasks) { task in
                            TaskCard(
                                task: task,
                                onToggle: { _Concurrency.Task { await session.toggleStatus(taskId: task.id) } },
                                onSkip: { _Concurrency.Task { await session.skip(taskId: task.id) } }
                            )
                            .transition(.asymmetric(
                                insertion: .scale(scale: 0.85).combined(with: .opacity).combined(with: .offset(y: 16)),
                                removal: .opacity
                            ))
                        }
                    }
                    if case .streaming = session.stashState {
                        streamingFooter
                    }
                    if case .done = session.stashState, !session.summary.isEmpty {
                        summaryFooter
                    }
                }
                .padding(20)
                .animation(.spring(response: 0.35, dampingFraction: 0.78), value: session.tasks.count)
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        router.navigate(.input)
                    } label: {
                        Image(systemName: "plus.circle.fill")
                            .font(.title2)
                            .foregroundStyle(Color.acornAccent)
                    }
                    .accessibilityLabel("New stash")
                }
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        router.navigate(.settings)
                    } label: {
                        Image(systemName: "gearshape")
                            .font(.title3)
                    }
                    .accessibilityLabel("Settings")
                }
            }
            .background(Color.warmCream.opacity(0.4).ignoresSafeArea())
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("Today")
                    .font(.acornTitle)
                Spacer()
                Text(progressLabel)
                    .font(.acornCaption.weight(.semibold))
                    .foregroundStyle(.secondary)
            }
            if let summary = session.currentSession?.aiSummary, !summary.isEmpty {
                Text(summary)
                    .font(.acornBody)
                    .foregroundStyle(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var progressLabel: String {
        let done = session.tasks.filter { $0.status == .completed || $0.status == .skipped }.count
        let total = session.tasks.count
        return total == 0 ? "" : "\(done)/\(total) done"
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            if case .awaitingFirstTask = session.stashState {
                ProgressView().controlSize(.large).tint(Color.acornAccent)
                Text("Thinking…")
                    .font(.acornHeader)
                    .foregroundStyle(.secondary)
            } else if case .streaming = session.stashState {
                ProgressView().controlSize(.large).tint(Color.acornAccent)
                Text("Decomposing…")
                    .font(.acornHeader)
                    .foregroundStyle(.secondary)
            } else if case .failed(let msg) = session.stashState {
                Image(systemName: "exclamationmark.triangle.fill")
                    .font(.largeTitle)
                    .foregroundStyle(Color.dustyRose)
                Text(msg)
                    .font(.acornBody)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            } else {
                Text("🌰")
                    .font(.system(size: 60))
                Text("Nothing stashed yet")
                    .font(.acornHeader)
                    .foregroundStyle(.secondary)
                Button("Stash something") {
                    router.navigate(.input)
                }
                .font(.acornButton)
                .foregroundStyle(Color.acornAccent)
            }
        }
        .frame(maxWidth: .infinity, minHeight: 240)
        .padding()
    }

    private var streamingFooter: some View {
        HStack(spacing: 8) {
            ProgressView().scaleEffect(0.7)
            Text("More on the way…")
                .font(.acornCaption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .center)
        .padding(.top, 8)
    }

    private var summaryFooter: some View {
        Text(session.summary)
            .font(.acornBody.italic())
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, 8)
    }
}
