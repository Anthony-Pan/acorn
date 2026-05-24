import Foundation
import Observation
#if canImport(AVFoundation)
import AVFoundation
#endif
import AcornCore

@MainActor
@Observable
public final class VoiceRecorder {
    public enum State: Sendable, Equatable {
        case idle
        case recording
        case transcribing
        case error(String)
    }

    public var state: State = .idle
    public private(set) var lastTranscript: String?

    private let speech: SpeechService
    #if canImport(AVFoundation)
    private var recorder: AVAudioRecorder?
    #endif
    private var fileURL: URL?

    public init(speech: SpeechService) {
        self.speech = speech
    }

    public func start() async {
        #if canImport(AVFoundation) && os(iOS)
        do {
            let session = AVAudioSession.sharedInstance()
            try session.setCategory(.playAndRecord, mode: .default, options: [.defaultToSpeaker, .allowBluetooth])
            try session.setActive(true)
            let granted = await Self.requestRecordPermission()
            guard granted else {
                state = .error("Microphone access denied. Grant in Settings → Acorn → Microphone.")
                return
            }
            let url = FileManager.default.temporaryDirectory
                .appendingPathComponent("acorn-record-\(UUID().uuidString).m4a")
            let settings: [String: Any] = [
                AVFormatIDKey: Int(kAudioFormatMPEG4AAC),
                AVSampleRateKey: 16_000,
                AVNumberOfChannelsKey: 1,
                AVEncoderAudioQualityKey: AVAudioQuality.medium.rawValue,
            ]
            let rec = try AVAudioRecorder(url: url, settings: settings)
            rec.prepareToRecord()
            rec.record()
            recorder = rec
            fileURL = url
            state = .recording
        } catch {
            state = .error("Couldn't start recording: \(error.localizedDescription)")
        }
        #else
        state = .error("Voice input requires iOS.")
        #endif
    }

    public func stopAndTranscribe(language: String?) async -> String? {
        #if canImport(AVFoundation) && os(iOS)
        guard let recorder, let url = fileURL, recorder.isRecording else {
            state = .idle
            return nil
        }
        recorder.stop()
        try? AVAudioSession.sharedInstance().setActive(false, options: [.notifyOthersOnDeactivation])
        state = .transcribing
        defer {
            try? FileManager.default.removeItem(at: url)
            self.recorder = nil
            self.fileURL = nil
        }
        do {
            let audio = try Data(contentsOf: url)
            let outcome = try await speech.transcribe(audio: audio, mimeType: "audio/m4a", language: language)
            lastTranscript = outcome.text
            state = .idle
            return outcome.text
        } catch {
            state = .error("Transcription failed: \(error.localizedDescription)")
            return nil
        }
        #else
        state = .idle
        return nil
        #endif
    }

    public func cancel() {
        #if canImport(AVFoundation) && os(iOS)
        recorder?.stop()
        if let url = fileURL { try? FileManager.default.removeItem(at: url) }
        try? AVAudioSession.sharedInstance().setActive(false, options: [.notifyOthersOnDeactivation])
        recorder = nil
        fileURL = nil
        #endif
        state = .idle
    }

    #if canImport(AVFoundation) && os(iOS)
    private static func requestRecordPermission() async -> Bool {
        if #available(iOS 17.0, *) {
            return await AVAudioApplication.requestRecordPermission()
        } else {
            return await withCheckedContinuation { cont in
                AVAudioSession.sharedInstance().requestRecordPermission { cont.resume(returning: $0) }
            }
        }
    }
    #endif
}
