import SwiftUI
import AcornCore

public struct TaskCard: View {
    let task: Task
    var onToggle: () -> Void
    var onSkip: () -> Void

    public init(
        task: Task,
        onToggle: @escaping () -> Void = {},
        onSkip: @escaping () -> Void = {}
    ) {
        self.task = task
        self.onToggle = onToggle
        self.onSkip = onSkip
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .top) {
                Button(action: onToggle) {
                    Image(systemName: iconForStatus)
                        .font(.title2)
                        .foregroundStyle(Color.priorityColor(task.priority))
                        .symbolEffect(.bounce, value: task.status)
                }
                .buttonStyle(.plain)

                VStack(alignment: .leading, spacing: 4) {
                    Text(task.title)
                        .font(.acornHeader)
                        .strikethrough(task.status == .completed || task.status == .skipped)
                        .foregroundStyle(
                            task.status == .completed || task.status == .skipped
                                ? Color.secondary
                                : Color.primary
                        )

                    if let desc = task.description, !desc.isEmpty {
                        Text(desc)
                            .font(.acornBody)
                            .foregroundStyle(.secondary)
                    }
                }

                Spacer(minLength: 0)
            }

            HStack(spacing: 8) {
                PriorityChip(task.priority)
                Label(durationLabel, systemImage: "clock")
                    .font(.acornCaption)
                    .foregroundStyle(.secondary)
                Spacer()
                if task.status == .pending {
                    Button("Skip", action: onSkip)
                        .font(.acornCaption.weight(.semibold))
                        .buttonStyle(.borderless)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .acornCard()
    }

    private var iconForStatus: String {
        switch task.status {
        case .pending: "circle"
        case .inProgress: "play.circle.fill"
        case .completed: "checkmark.circle.fill"
        case .skipped: "arrowshape.right.circle.fill"
        }
    }

    private var durationLabel: String {
        let m = task.durationMinutes
        if m < 60 { return "\(m) min" }
        let h = m / 60
        let r = m % 60
        return r == 0 ? "\(h) hr" : "\(h)h \(r)m"
    }
}
