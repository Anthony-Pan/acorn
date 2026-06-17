import ActivityKit
import WidgetKit
import SwiftUI
import AcornCore

@available(iOS 16.2, *)
struct TaskActivityWidget: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: TaskActivityAttributes.self) { context in
            LockScreenView(context: context)
                .activityBackgroundTint(Color.black.opacity(0.6))
                .activitySystemActionForegroundColor(.white)
        } dynamicIsland: { context in
            DynamicIsland {
                DynamicIslandExpandedRegion(.leading) {
                    Text("🌰")
                        .font(.title2)
                }
                DynamicIslandExpandedRegion(.center) {
                    Text(context.attributes.title)
                        .font(.callout.weight(.semibold))
                        .lineLimit(2)
                }
                DynamicIslandExpandedRegion(.trailing) {
                    Text(durationLabel(elapsed: context.state.elapsedSeconds, plannedMinutes: context.attributes.durationMinutes))
                        .font(.caption.monospacedDigit())
                }
                DynamicIslandExpandedRegion(.bottom) {
                    HStack(spacing: 6) {
                        Circle().fill(priorityColor(context.attributes.priority)).frame(width: 6, height: 6)
                        Text(statusLabel(context.state.status))
                            .font(.caption.weight(.semibold))
                        Spacer()
                    }
                }
            } compactLeading: {
                Text("🌰")
            } compactTrailing: {
                Text(durationLabel(elapsed: context.state.elapsedSeconds, plannedMinutes: context.attributes.durationMinutes))
                    .font(.caption2.monospacedDigit())
            } minimal: {
                Text("🌰")
            }
        }
    }
}

@available(iOS 16.2, *)
struct LockScreenView: View {
    let context: ActivityViewContext<TaskActivityAttributes>

    var body: some View {
        HStack(spacing: 12) {
            Text("🌰")
                .font(.title)
            VStack(alignment: .leading, spacing: 2) {
                Text(context.attributes.title)
                    .font(.headline)
                    .lineLimit(1)
                HStack(spacing: 6) {
                    Circle().fill(priorityColor(context.attributes.priority)).frame(width: 6, height: 6)
                    Text(statusLabel(context.state.status))
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                }
            }
            Spacer()
            Text(durationLabel(elapsed: context.state.elapsedSeconds, plannedMinutes: context.attributes.durationMinutes))
                .font(.title3.monospacedDigit().weight(.semibold))
        }
        .padding()
    }
}

@available(iOS 16.2, *)
private func durationLabel(elapsed: Int, plannedMinutes: Int) -> String {
    let m = elapsed / 60
    let s = elapsed % 60
    return String(format: "%d:%02d / %dm", m, s, plannedMinutes)
}

@available(iOS 16.2, *)
private func statusLabel(_ s: TaskStatus) -> String {
    switch s {
    case .pending: "Pending"
    case .inProgress: "In progress"
    case .completed: "Done"
    case .skipped: "Skipped"
    }
}

@available(iOS 16.2, *)
private func priorityColor(_ p: Priority) -> Color {
    switch p {
    case .high: Color(red: 0.788, green: 0.471, blue: 0.443)
    case .medium: Color(red: 0.910, green: 0.667, blue: 0.310)
    case .low: Color(red: 0.404, green: 0.502, blue: 0.282)
    }
}
