import SwiftUI
import AcornCore

public struct AILanguagePicker: View {
    @Environment(SettingsStore.self) private var settings
    @Environment(\.dismiss) private var dismiss

    public init() {}

    public var body: some View {
        List {
            Section {
                ForEach(SettingsStore.availableAILanguages(), id: \.code) { entry in
                    Button(action: { choose(entry.code) }) {
                        HStack {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(entry.displayName)
                                    .font(.acornBody.weight(.semibold))
                                Text(entry.code)
                                    .font(.acornCaption.monospaced())
                                    .foregroundStyle(.secondary)
                            }
                            Spacer()
                            if settings.language == entry.code {
                                Image(systemName: "checkmark.circle.fill")
                                    .foregroundStyle(Color.acornAccent)
                            }
                        }
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                }
            } footer: {
                Text("Acorn tells the AI to reply in this language. Cloud providers, FoundationModels, and on-device speech all respect it.")
                    .font(.acornCaption)
            }
        }
        .navigationTitle("AI language")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }

    private func choose(_ code: String) {
        _Concurrency.Task { await settings.setLanguage(code); dismiss() }
    }
}
