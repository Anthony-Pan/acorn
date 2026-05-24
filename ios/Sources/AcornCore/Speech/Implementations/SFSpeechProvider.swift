import Foundation

#if canImport(Speech)
import Speech

public actor SFSpeechProvider: SpeechProvider {
    public let metadata: SpeechProviderMetadata

    public init(metadata: SpeechProviderMetadata) {
        self.metadata = metadata
    }

    public func transcribe(audio: Data, mimeType: String, language: String?) async throws -> TranscribeOutcome {
        try await Self.requestAuthorizationIfNeeded()
        let locale = language.flatMap(Locale.init(identifier:)) ?? Locale.current
        guard let recognizer = SFSpeechRecognizer(locale: locale), recognizer.isAvailable else {
            throw ProviderError.modelUnavailable(
                reason: "Speech recognition not available for \(locale.identifier)."
            )
        }
        let tempURL = try Self.writeTempAudio(audio: audio, mimeType: mimeType)
        defer { try? FileManager.default.removeItem(at: tempURL) }
        let request = SFSpeechURLRecognitionRequest(url: tempURL)
        request.shouldReportPartialResults = false
        if #available(iOS 17.0, macOS 14.0, *) {
            request.addsPunctuation = true
        }
        return try await withCheckedThrowingContinuation { continuation in
            recognizer.recognitionTask(with: request) { result, error in
                if let error {
                    continuation.resume(throwing: ProviderError.providerResponse(error.localizedDescription))
                    return
                }
                guard let result, result.isFinal else { return }
                let outcome = TranscribeOutcome(
                    text: result.bestTranscription.formattedString,
                    providerId: "system"
                )
                continuation.resume(returning: outcome)
            }
        }
    }

    private static func requestAuthorizationIfNeeded() async throws {
        let status = SFSpeechRecognizer.authorizationStatus()
        if status == .authorized { return }
        let granted: SFSpeechRecognizerAuthorizationStatus = await withCheckedContinuation { cont in
            SFSpeechRecognizer.requestAuthorization { cont.resume(returning: $0) }
        }
        guard granted == .authorized else {
            throw ProviderError.modelUnavailable(
                reason: "Speech recognition permission denied. Grant access in Settings → Privacy → Speech Recognition."
            )
        }
    }

    private static func writeTempAudio(audio: Data, mimeType: String) throws -> URL {
        let ext = mimeType.split(separator: "/").last.map(String.init) ?? "m4a"
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent("acorn-speech-\(UUID().uuidString).\(ext)")
        try audio.write(to: url)
        return url
    }
}

#endif
