import Foundation

public struct OpenAICompatibleProvider: Provider {
    public let metadata: ProviderMetadata
    public let supportsTools: Bool = true

    private let apiKey: String
    private let model: String
    private let endpoint: URL
    private let session: URLSession

    public init(
        metadata: ProviderMetadata,
        apiKey: String,
        model: String? = nil,
        customEndpoint: String? = nil,
        session: URLSession = .shared
    ) {
        self.metadata = metadata
        self.apiKey = apiKey
        self.model = model ?? metadata.defaultModel
        self.endpoint = URL(string: customEndpoint ?? metadata.defaultEndpoint)!
        self.session = session
    }

    public func validateCredentials() async throws {
        let probe = DecomposeRequest(rawInput: "ping", language: "en")
        _ = try await callOnce(probe, temperature: 0.0, maxTokens: 16)
    }

    public func decompose(_ request: DecomposeRequest) -> AsyncThrowingStream<DecomposeEvent, Error> {
        AsyncThrowingStream { continuation in
            _Concurrency.Task {
                do {
                    let raw = try await callOnce(request, temperature: 0.3, maxTokens: 2048)
                    let parsed = try DecomposeParser.parse(raw)
                    await DecomposeEmitter.emit(response: parsed, to: continuation)
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }

    private func callOnce(_ request: DecomposeRequest, temperature: Double, maxTokens: Int) async throws -> String {
        let useJSONMode = Self.supportsJSONMode(metadata.id)
        let body = OpenAIRequestBody(
            model: model,
            messages: [
                .init(role: "system", content: DecomposePrompts.systemPrompt(language: request.language)),
                .init(role: "user", content: DecomposePrompts.userPrompt(request)),
            ],
            temperature: temperature,
            maxTokens: maxTokens,
            responseFormat: useJSONMode ? .jsonObject : nil
        )
        var req = URLRequest(url: endpoint)
        req.httpMethod = "POST"
        req.addValue("application/json", forHTTPHeaderField: "content-type")
        req.addValue("Bearer \(apiKey)", forHTTPHeaderField: "authorization")
        req.httpBody = try JSONEncoder().encode(body)

        let (data, response) = try await session.data(for: req)
        guard let http = response as? HTTPURLResponse else {
            throw ProviderError.network("No HTTP response.")
        }
        if http.statusCode == 401 {
            throw ProviderError.invalidKey
        }
        if http.statusCode == 429 {
            throw ProviderError.rateLimited
        }
        if !(200..<300).contains(http.statusCode) {
            let body = String(data: data, encoding: .utf8) ?? ""
            throw ProviderError.providerResponse("HTTP \(http.statusCode) from \(metadata.displayName): \(body)")
        }
        let decoded = try JSONDecoder().decode(OpenAIResponseBody.self, from: data)
        guard let content = decoded.choices.first?.message.content, !content.isEmpty else {
            throw ProviderError.invalidResponse("Empty response from \(metadata.displayName).")
        }
        return content
    }

    private static func supportsJSONMode(_ id: String) -> Bool {
        switch id {
        case "openai", "deepseek", "openrouter", "groq", "mistral", "moonshot", "qwen":
            return true
        default:
            return false
        }
    }
}

private struct OpenAIRequestBody: Codable {
    let model: String
    let messages: [OpenAIMessage]
    let temperature: Double
    let maxTokens: Int
    let responseFormat: OpenAIResponseFormat?

    enum CodingKeys: String, CodingKey {
        case model
        case messages
        case temperature
        case maxTokens = "max_tokens"
        case responseFormat = "response_format"
    }
}

private struct OpenAIMessage: Codable {
    let role: String
    let content: String
}

private struct OpenAIResponseFormat: Codable {
    let type: String
    static let jsonObject = OpenAIResponseFormat(type: "json_object")
}

private struct OpenAIResponseBody: Codable {
    let choices: [OpenAIChoice]
}

private struct OpenAIChoice: Codable {
    let message: OpenAIResponseMessage
}

private struct OpenAIResponseMessage: Codable {
    let content: String
}
