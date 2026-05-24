import SwiftUI
import AcornCore

public struct VoiceButton: View {
    @State private var recorder: VoiceRecorder
    var language: String
    var onTranscript: (String) -> Void

    public init(speech: SpeechService, language: String, onTranscript: @escaping (String) -> Void) {
        _recorder = State(initialValue: VoiceRecorder(speech: speech))
        self.language = language
        self.onTranscript = onTranscript
    }

    public var body: some View {
        Button {
            _Concurrency.Task { @MainActor in
                switch recorder.state {
                case .idle, .error:
                    await recorder.start()
                case .recording:
                    if let transcript = await recorder.stopAndTranscribe(language: language),
                       !transcript.isEmpty {
                        onTranscript(transcript)
                    }
                case .transcribing:
                    break
                }
            }
        } label: {
            ZStack {
                Circle()
                    .fill(backgroundColor)
                    .frame(width: 48, height: 48)
                    .shadow(color: Color.acornAccent.opacity(0.35), radius: 8, x: 0, y: 4)
                if case .transcribing = recorder.state {
                    ProgressView().tint(.white)
                } else {
                    Image(systemName: iconName)
                        .font(.title3.weight(.semibold))
                        .foregroundStyle(.white)
                        .symbolEffect(.bounce, value: recorder.state == .recording)
                }
            }
        }
        .buttonStyle(.plain)
        .accessibilityLabel(accessibilityLabel)
        .overlay(alignment: .top) {
            if case .error(let msg) = recorder.state {
                Text(msg)
                    .font(.acornCaption)
                    .foregroundStyle(Color.dustyRose)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(.regularMaterial, in: Capsule())
                    .offset(y: -48)
                    .frame(maxWidth: 240)
            }
        }
    }

    private var backgroundColor: Color {
        switch recorder.state {
        case .recording: Color.dustyRose
        case .transcribing: Color.autumnAmber
        default: Color.acornAccent
        }
    }

    private var iconName: String {
        switch recorder.state {
        case .recording: "stop.fill"
        default: "mic.fill"
        }
    }

    private var accessibilityLabel: String {
        switch recorder.state {
        case .recording: "Stop recording"
        case .transcribing: "Transcribing"
        default: "Voice input"
        }
    }
}
