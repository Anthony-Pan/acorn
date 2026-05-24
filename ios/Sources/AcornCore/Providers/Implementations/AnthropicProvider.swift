import Foundation

public struct AnthropicProvider: Provider {
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
        _ = try await callOnce(probe, maxTokens: 16, temperature: 0.0)
    }

    public func decompose(_ request: DecomposeRequest) -> AsyncThrowingStream<DecomposeEvent, Error> {
        AsyncThrowingStream { continuation in
            _Concurrency.Task {
                do {
                    let raw = try await callOnce(request, maxTokens: 2048, temperature: 0.3)
                    let parsed = try DecomposeParser.parse(raw)
                    await DecomposeEmitter.emit(response: parsed, to: continuation)
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }

    private func callOnce(_ request: DecomposeRequest, maxTokens: Int, temperature: Double) async throws -> String {
        let body = AnthropicRequestBody(
            model: model,
            maxTokens: maxTokens,
            system: DecomposePrompts.systemPrompt(language: request.language),
            messages: [
                AnthropicMessage(role: "user", content: DecomposePrompts.userPrompt(request)),
            ],
            temperature: temperature
        )
        var req = URLRequest(url: endpoint)
        req.httpMethod = "POST"
        req.addValue("application/json", forHTTPHeaderField: "content-type")
        req.addValue(apiKey, forHTTPHeaderField: "x-api-key")
        req.addValue("2023-06-01", forHTTPHeaderField: "anthropic-version")
        req.httpBody = try JSONEncoder().encode(body)

        let (data, response) = try await session.data(for: req)
        guard let http = response as? HTTPURLResponse else {
            throw ProviderError.network("No HTTP response from Anthropic.")
        }
        if http.statusCode == 401 {
            throw ProviderError.invalidKey
        }
        if http.statusCode == 429 {
            throw ProviderError.rateLimited
        }
        if !(200..<300).contains(http.statusCode) {
            let body = String(data: data, encoding: .utf8) ?? ""
            throw ProviderError.providerResponse("Anthropic HTTP \(http.statusCode): \(body)")
        }
        let decoded = try JSONDecoder().decode(AnthropicResponse.self, from: data)
        let text = decoded.content.compactMap { block -> String? in
            if case .text(let t) = block { return t }
            return nil
        }.joined(separator: "\n")
        return text
    }
}

private struct AnthropicRequestBody: Codable {
    let model: String
    let maxTokens: Int
    let system: String
    let messages: [AnthropicMessage]
    let temperature: Double

    enum CodingKeys: String, CodingKey {
        case model
        case maxTokens = "max_tokens"
        case system
        case messages
        case temperature
    }
}

private struct AnthropicMessage: Codable {
    let role: String
    let content: String
}

private struct AnthropicResponse: Codable {
    let content: [AnthropicContent]
}

private enum AnthropicContent: Codable {
    case text(String)
    case other

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        if type == "text" {
            let text = try container.decode(String.self, forKey: .text)
            self = .text(text)
        } else {
            self = .other
        }
    }

    func encode(to encoder: Encoder) throws {}

    private enum CodingKeys: String, CodingKey {
        case type
        case text
    }
}
