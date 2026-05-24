import SwiftUI
import AcornCore

public struct InputView: View {
    @Environment(SessionStore.self) private var session
    @Environment(SettingsStore.self) private var settings
    @Environment(ProvidersStore.self) private var providers
    @Environment(AppRouter.self) private var router

    let speechService: SpeechService

    @State private var text: String = ""
    @State private var isWorking = false
    @State private var errorMessage: String?
    @FocusState private var editorFocused: Bool

    public init(speechService: SpeechService) {
        self.speechService = speechService
    }

    public var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 16) {
                header
                editor
                providerRow
                stashButton
                if let err = errorMessage {
                    Text(err)
                        .font(.acornCaption)
                        .foregroundStyle(Color.dustyRose)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                Spacer(minLength: 0)
            }
            .padding(20)
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        router.navigate(.settings)
                    } label: {
                        Image(systemName: "gearshape")
                            .font(.title3)
                    }
                    .accessibilityLabel("Settings")
                }
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        router.navigate(.history)
                    } label: {
                        Image(systemName: "clock.arrow.circlepath")
                            .font(.title3)
                    }
                    .accessibilityLabel("History")
                }
            }
            .background(Color.warmCream.opacity(0.4).ignoresSafeArea())
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("🌰")
                    .font(.system(size: 32))
                Text("Stash your day")
                    .font(.acornTitle)
            }
            Text("Dump everything on your mind — Acorn will break it into focused task cards.")
                .font(.acornBody)
                .foregroundStyle(.secondary)
        }
    }

    private var editor: some View {
        ZStack(alignment: .topLeading) {
            if text.isEmpty {
                Text("call the dentist, finish the slide deck, look at the launch metrics, run by the bank…")
                    .font(.acornBody)
                    .foregroundStyle(.tertiary)
                    .padding(.top, 14)
                    .padding(.leading, 14)
                    .allowsHitTesting(false)
            }
            TextEditor(text: $text)
                .focused($editorFocused)
                .font(.acornBody)
                .scrollContentBackground(.hidden)
                .padding(8)
        }
        .frame(minHeight: 220)
        .acornCard()
    }

    private var providerRow: some View {
        HStack(spacing: 8) {
            Image(systemName: "sparkles")
                .foregroundStyle(Color.acornAccent)
            if let metadata = providers.metadata(for: settings.activeProviderId) {
                Text("Using")
                    .font(.acornCaption)
                    .foregroundStyle(.secondary)
                Text(metadata.displayName)
                    .font(.acornCaption.weight(.semibold))
            } else {
                Text("Pick a provider in Settings")
                    .font(.acornCaption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button {
                router.navigate(.settings)
            } label: {
                Text("Change")
                    .font(.acornCaption.weight(.semibold))
            }
            .buttonStyle(.borderless)
        }
        .padding(.horizontal, 4)
    }

    private var stashButton: some View {
        HStack(spacing: 12) {
            Button {
                stash()
            } label: {
                HStack {
                    Spacer()
                    if isWorking {
                        ProgressView().tint(.white)
                    } else {
                        Text("Stash it 🌰")
                            .font(.acornButton)
                    }
                    Spacer()
                }
                .padding(.vertical, 16)
                .background(
                    RoundedRectangle(cornerRadius: 14, style: .continuous)
                        .fill(Color.acornAccent.gradient)
                )
                .foregroundStyle(.white)
                .shadow(color: Color.acornAccent.opacity(0.45), radius: 12, x: 0, y: 5)
            }
            .buttonStyle(.plain)
            .disabled(text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || isWorking)

            VoiceButton(
                speech: speechService,
                language: settings.speechLanguage ?? settings.language
            ) { transcript in
                if text.isEmpty {
                    text = transcript
                } else {
                    text += "\n" + transcript
                }
            }
        }
    }

    private func stash() {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        errorMessage = nil
        isWorking = true
        editorFocused = false
        let payload = text
        let language = settings.language.isEmpty ? "en" : settings.language
        let providerId = settings.activeProviderId
        router.navigate(.stash)
        _Concurrency.Task {
            await session.stash(rawInput: payload, language: language, providerId: providerId)
            isWorking = false
            text = ""
            if case .failed(let msg) = session.stashState {
                errorMessage = msg
                router.navigate(.input)
            }
        }
    }
}
