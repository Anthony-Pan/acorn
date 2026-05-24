import WidgetKit
import SwiftUI
import AcornCore

struct TodayEntry: TimelineEntry {
    let date: Date
    let snapshot: WidgetSnapshot
}

struct TodayProvider: TimelineProvider {
    func placeholder(in context: Context) -> TodayEntry {
        TodayEntry(
            date: Date(),
            snapshot: WidgetSnapshot(
                summary: "Stash your day in Acorn.",
                tasks: [
                    .init(id: "1", title: "Pick up dry cleaning", priority: .high, durationMinutes: 15, status: .pending),
                    .init(id: "2", title: "Finish the slide deck", priority: .medium, durationMinutes: 45, status: .pending),
                    .init(id: "3", title: "Call the dentist", priority: .low, durationMinutes: 10, status: .pending),
                ]
            )
        )
    }

    func getSnapshot(in context: Context, completion: @escaping (TodayEntry) -> Void) {
        completion(TodayEntry(date: Date(), snapshot: WidgetCache.read()))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<TodayEntry>) -> Void) {
        let snapshot = WidgetCache.read()
        let entry = TodayEntry(date: Date(), snapshot: snapshot)
        let refresh = Date().addingTimeInterval(15 * 60)
        completion(Timeline(entries: [entry], policy: .after(refresh)))
    }
}

struct TodayWidget: Widget {
    let kind = "TodayWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: TodayProvider()) { entry in
            TodayWidgetView(entry: entry)
                .containerBackground(.background, for: .widget)
        }
        .configurationDisplayName("Today in Acorn")
        .description("See your stashed tasks for today.")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge, .accessoryRectangular])
    }
}

struct TodayWidgetView: View {
    let entry: TodayEntry
    @Environment(\.widgetFamily) var family

    var body: some View {
        switch family {
        case .systemSmall: smallView
        case .systemMedium: mediumView
        case .systemLarge: largeView
        case .accessoryRectangular: lockScreenView
        default: smallView
        }
    }

    private var smallView: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("🌰")
                Text("Today")
                    .font(.headline)
                Spacer()
            }
            Spacer()
            if let first = entry.snapshot.tasks.first {
                Text(first.title)
                    .font(.callout.weight(.semibold))
                    .lineLimit(3)
                if entry.snapshot.tasks.count > 1 {
                    Text("+\(entry.snapshot.tasks.count - 1) more")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            } else {
                Text("Nothing stashed yet.")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }

    private var mediumView: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 6) {
                Text("🌰")
                Text("Today in Acorn")
                    .font(.headline)
                Spacer()
                Text(timeLabel)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            ForEach(entry.snapshot.tasks.prefix(4)) { task in
                HStack(spacing: 6) {
                    Circle()
                        .fill(color(for: task.priority))
                        .frame(width: 6, height: 6)
                    Text(task.title)
                        .font(.caption)
                        .lineLimit(1)
                    Spacer()
                }
            }
            if entry.snapshot.tasks.isEmpty {
                Text("Nothing stashed yet.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var largeView: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 6) {
                Text("🌰")
                Text("Today in Acorn")
                    .font(.headline)
                Spacer()
                Text(timeLabel)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            if !entry.snapshot.summary.isEmpty {
                Text(entry.snapshot.summary)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }
            Divider()
            ForEach(entry.snapshot.tasks.prefix(8)) { task in
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Circle()
                        .fill(color(for: task.priority))
                        .frame(width: 6, height: 6)
                    VStack(alignment: .leading) {
                        Text(task.title)
                            .font(.caption.weight(.semibold))
                            .lineLimit(1)
                        Text("\(task.durationMinutes) min")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    Image(systemName: icon(for: task.status))
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
            if entry.snapshot.tasks.isEmpty {
                Text("Nothing stashed yet.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
        }
    }

    private var lockScreenView: some View {
        VStack(alignment: .leading, spacing: 1) {
            if let first = entry.snapshot.tasks.first {
                Text(first.title)
                    .font(.caption2.weight(.semibold))
                    .lineLimit(1)
                if entry.snapshot.tasks.count > 1 {
                    Text("+\(entry.snapshot.tasks.count - 1) more")
                        .font(.caption2)
                }
            } else {
                Text("🌰 Acorn")
                    .font(.caption2.weight(.semibold))
            }
        }
    }

    private var timeLabel: String {
        let f = DateFormatter()
        f.dateStyle = .none
        f.timeStyle = .short
        return f.string(from: entry.snapshot.updatedAt)
    }

    private func color(for priority: Priority) -> Color {
        switch priority {
        case .high: return Color(red: 0.788, green: 0.471, blue: 0.443)
        case .medium: return Color(red: 0.910, green: 0.667, blue: 0.310)
        case .low: return Color(red: 0.404, green: 0.502, blue: 0.282)
        }
    }

    private func icon(for status: TaskStatus) -> String {
        switch status {
        case .pending: "circle"
        case .inProgress: "play.circle.fill"
        case .completed: "checkmark.circle.fill"
        case .skipped: "arrowshape.right.circle.fill"
        }
    }
}

#Preview(as: .systemMedium) {
    TodayWidget()
} timeline: {
    TodayEntry(date: .now, snapshot: WidgetSnapshot(
        summary: "Three things to ship before lunch.",
        tasks: [
            .init(id: "1", title: "Pick up dry cleaning", priority: .high, durationMinutes: 15, status: .pending),
            .init(id: "2", title: "Finish the slide deck", priority: .medium, durationMinutes: 45, status: .inProgress),
            .init(id: "3", title: "Call the dentist", priority: .low, durationMinutes: 10, status: .completed),
        ]
    ))
}
