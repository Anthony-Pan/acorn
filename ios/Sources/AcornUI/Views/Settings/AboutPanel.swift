import SwiftUI
import AcornCore

public struct AboutPanel: View {
    public init() {}

    public var body: some View {
        Form {
            Section {
                VStack(alignment: .leading, spacing: 8) {
                    Text("🌰 Acorn iOS")
                        .font(.acornTitle)
                    Text("A warm, model-agnostic AI companion that turns a messy day into focused task cards.")
                        .font(.acornBody)
                        .foregroundStyle(.secondary)
                }
                .padding(.vertical, 4)
            }

            Section("Build") {
                LabeledContent("Version") {
                    Text(AcornVersion.current).foregroundStyle(.secondary)
                }
                LabeledContent("macOS parity") {
                    Text(AcornVersion.macOSEquivalent).foregroundStyle(.secondary)
                }
                LabeledContent("Bundle id") {
                    Text(Bundle.main.bundleIdentifier ?? "—")
                        .font(.acornCaption.monospaced())
                        .foregroundStyle(.secondary)
                }
            }

            Section("Source") {
                Link(destination: URL(string: "https://github.com/onyxcraft/acorn")!) {
                    Label("Repository", systemImage: "chevron.left.forwardslash.chevron.right")
                }
                Link(destination: URL(string: "https://github.com/onyxcraft/acorn/blob/main/LICENSE")!) {
                    Label("MIT License", systemImage: "scroll")
                }
            }

            Section("Acknowledgments") {
                Text("Built with SwiftUI, GRDB.swift, and Apple FoundationModels. Voice input via Whisper or on-device SFSpeechRecognizer.")
                    .font(.acornCaption)
                    .foregroundStyle(.secondary)
            }
        }
        .navigationTitle("About")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }
}
