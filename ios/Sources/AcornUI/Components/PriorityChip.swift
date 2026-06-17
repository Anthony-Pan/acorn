import SwiftUI
import AcornCore

public struct PriorityChip: View {
    let priority: Priority

    public init(_ priority: Priority) {
        self.priority = priority
    }

    public var body: some View {
        HStack(spacing: 4) {
            Circle()
                .fill(Color.priorityColor(priority))
                .frame(width: 8, height: 8)
            Text(label)
                .font(.acornCaption.weight(.semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .tracking(0.6)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(
            Capsule()
                .fill(Color.priorityColor(priority).opacity(0.12))
        )
    }

    private var label: String {
        switch priority {
        case .high: "High"
        case .medium: "Medium"
        case .low: "Low"
        }
    }
}

#Preview {
    HStack { PriorityChip(.high); PriorityChip(.medium); PriorityChip(.low) }
        .padding()
}
