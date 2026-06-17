import Foundation

public struct WhisperProvider: SpeechProvider {
    public let metadata: SpeechProviderMetadata
    private let apiKey: String
    private let session: URLSession
    private let endpoint = URL(string: "https://api.openai.com/v1/audio/transcriptions")!

    public init(metadata: SpeechProviderMetadata, apiKey: String, session: URLSession = .shared) {
        self.metadata = metadata
        self.apiKey = apiKey
        self.session = session
    }

    public func transcribe(audio: Data, mimeType: String, language: String?) async throws -> TranscribeOutcome {
        let boundary = "AcornBoundary\(UUID().uuidString)"
        var req = URLRequest(url: endpoint)
        req.httpMethod = "POST"
        req.addValue("Bearer \(apiKey)", forHTTPHeaderField: "authorization")
        req.addValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "content-type")

        var body = Data()
        let crlf = "\r\n"
        func append(_ s: String) {
            body.append(s.data(using: .utf8)!)
        }
        let filename = "audio." + (mimeType.split(separator: "/").last.map(String.init) ?? "m4a")

        append("--\(boundary)\(crlf)")
        append("Content-Disposition: form-data; name=\"model\"\(crlf)\(crlf)whisper-1\(crlf)")

        if let language, !language.isEmpty {
            append("--\(boundary)\(crlf)")
            append("Content-Disposition: form-data; name=\"language\"\(crlf)\(crlf)\(language)\(crlf)")
        }

        append("--\(boundary)\(crlf)")
        append("Content-Disposition: form-data; name=\"file\"; filename=\"\(filename)\"\(crlf)")
        append("Content-Type: \(mimeType)\(crlf)\(crlf)")
        body.append(audio)
        append(crlf)
        append("--\(boundary)--\(crlf)")

        req.httpBody = body

        let (data, response) = try await session.data(for: req)
        guard let http = response as? HTTPURLResponse else {
            throw ProviderError.network("No HTTP response from Whisper.")
        }
        if http.statusCode == 401 {
            throw ProviderError.invalidKey
        }
        if !(200..<300).contains(http.statusCode) {
            let s = String(data: data, encoding: .utf8) ?? ""
            throw ProviderError.providerResponse("Whisper HTTP \(http.statusCode): \(s)")
        }
        let decoded = try JSONDecoder().decode(WhisperResponse.self, from: data)
        return TranscribeOutcome(text: decoded.text, providerId: metadata.id)
    }
}

private struct WhisperResponse: Decodable {
    let text: String
}
