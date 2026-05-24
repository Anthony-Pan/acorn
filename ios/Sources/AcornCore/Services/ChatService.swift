import Foundation

public actor ChatService {
    private let conversations: ConversationService
    private let providerConfigs: ProviderConfigService
    private let memory: MemoryService
    private let settings: SettingsService
    private let activity: ActivityService

    public init(
        conversations: ConversationService,
        providerConfigs: ProviderConfigService,
        memory: MemoryService,
        settings: SettingsService,
        activity: ActivityService
    ) {
        self.conversations = conversations
        self.providerConfigs = providerConfigs
        self.memory = memory
        self.settings = settings
        self.activity = activity
    }

    public func send(
        conversationId: String,
        providerId: String,
        userText: String
    ) async throws -> Message {
        let trimmed = userText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { throw ChatRequestError.empty }

        guard let convo = try await conversations.get(conversationId) else {
            throw ProviderError.invalidResponse("Conversation \(conversationId) missing.")
        }
        if let locked = convo.providerId, locked != providerId {
            throw ChatRequestError.providerMismatch(locked: locked, attempted: providerId)
        }

        _ = try await conversations.appendMessage(
            conversationId: conversationId,
            role: .user,
            content: trimmed
        )
        try? await activity.record(kind: .chat, content: trimmed)

        let provider = try await makeProvider(id: providerId)
        let history = try await conversations.messages(for: conversationId)
        let turns = history.compactMap { msg -> ChatTurn? in
            switch msg.role {
            case .user: return ChatTurn(role: "user", content: msg.content)
            case .assistant: return ChatTurn(role: "assistant", content: msg.content)
            default: return nil
            }
        }

        let language = (try? await settings.get(.language)) ?? "en"
        let sharedMemory = (try? await memory.read()) ?? ""
        let systemPrompt = Self.composeChatSystemPrompt(language: language, sharedMemory: sharedMemory)

        let reply = try await provider.chat(turns: turns, systemPrompt: systemPrompt, temperature: 0.4)
        let assistantMessage = try await conversations.appendMessage(
            conversationId: conversationId,
            role: .assistant,
            content: reply
        )
        if convo.providerId == nil {
            _ = try await conversations.setProvider(
                conversationId,
                providerId: providerId,
                model: provider.metadata.defaultModel
            )
        }
        return assistantMessage
    }

    private func makeProvider(id providerId: String) async throws -> any Provider {
        guard let metadata = ProviderCatalog.get(providerId) else {
            throw ProviderError.notImplemented("Unknown provider id: \(providerId)")
        }
        var apiKey: String?
        if metadata.requiresApiKey {
            apiKey = try await providerConfigs.readCredentials(providerId: providerId)
            if apiKey == nil || apiKey?.isEmpty == true {
                throw ProviderError.missingCredentials(providerId)
            }
        }
        let config = try await providerConfigs.get(providerId)
        return try ProviderFactory.build(ProviderInputs(
            metadata: metadata,
            apiKey: apiKey,
            model: config?.selectedModel,
            customEndpoint: config?.customEndpoint
        ))
    }

    private static func composeChatSystemPrompt(language: String, sharedMemory: String) -> String {
        let lang = DecomposePrompts.languageDisplayName(for: language)
        var prompt = """
        You are Acorn, a warm, focused companion. Keep replies short, kind, and useful. Format any structured suggestions as plain markdown.
        """
        let trimmedMemory = sharedMemory.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmedMemory.isEmpty {
            prompt += "\n\nWhat your friend wants you to keep in mind:\n\(trimmedMemory)"
        }
        prompt += "\n\nYour friend has set the app language to \(lang). Default to replying in that language unless they switch on their own."
        return prompt
    }
}
