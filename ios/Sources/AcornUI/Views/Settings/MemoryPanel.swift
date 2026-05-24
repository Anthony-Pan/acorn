import SwiftUI
import AcornCore

public struct MemoryPanel: View {
    @Environment(SettingsStore.self) private var settings
    @State private var draft: String = ""
    @State private var initial: String = ""
    @State private var showCleared = false

    public init() {}

    public var body: some View {
        Form {
            Section {
                TextEditor(text: $draft)
                    .frame(minHeight: 220)
                    .font(.acornBody)
            } header: {
                Text("Shared memory")
            } footer: {
                Text("Acorn shares this with every AI provider on every stash. Keep it factual and short — long-term context, allergies, time zone, working hours, etc.")
                    .font(.acornCaption)
            }

            Section {
                Button {
                    save()
                } label: {
                    Label("Save", systemImage: "tray.and.arrow.down")
                }
                .disabled(draft == initial)

                Button(role: .destructive) {
                    clear()
                } label: {
                    Label("Clear memory", systemImage: "trash")
                }
                .disabled(draft.isEmpty && initial.isEmpty)
            }

            if showCleared {
                Text("Cleared.")
                    .font(.acornCaption)
                    .foregroundStyle(Color.mossGreen)
            }
        }
        .navigationTitle("Memory")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task {
            draft = settings.sharedMemory
            initial = settings.sharedMemory
        }
    }

    private func save() {
        let value = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        _Concurrency.Task {
            await settings.setSharedMemory(value)
            initial = value
            draft = value
        }
    }

    private func clear() {
        _Concurrency.Task {
            await settings.setSharedMemory("")
            draft = ""
            initial = ""
            showCleared = true
            try? await _Concurrency.Task.sleep(for: .seconds(1.5))
            showCleared = false
        }
    }
}
