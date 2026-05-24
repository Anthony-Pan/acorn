import Foundation
import Observation
import AcornCore

@MainActor
@Observable
public final class ChatStore {
    public var conversations: [Conversation] = []
    public var currentConversation: Conversation?
    public var messages: [Message] = []
    public var sending: Bool = false
    public var error: String?

    private let services: Services

    public init(services: Services) {
        self.services = services
    }

    public func hydrate() async {
        conversations = (try? await services.conversations.list()) ?? []
        if currentConversation == nil, let first = conversations.first {
            await select(first.id)
        }
    }

    public func select(_ conversationId: String) async {
        if let convo = try? await services.conversations.get(conversationId) {
            currentConversation = convo
            messages = (try? await services.conversations.messages(for: conversationId)) ?? []
            error = nil
        }
    }

    public func createNew() async {
        guard let convo = try? await services.conversations.create(title: nil) else { return }
        conversations.insert(convo, at: 0)
        currentConversation = convo
        messages = []
    }

    public func deleteCurrent() async {
        guard let id = currentConversation?.id else { return }
        try? await services.conversations.delete(id)
        conversations.removeAll(where: { $0.id == id })
        currentConversation = conversations.first
        if let next = currentConversation {
            await select(next.id)
        } else {
            messages = []
        }
    }

    public func rename(_ id: String, to title: String) async {
        guard let updated = try? await services.conversations.rename(id, title: title) else { return }
        if let idx = conversations.firstIndex(where: { $0.id == id }) {
            conversations[idx] = updated
        }
        if currentConversation?.id == id {
            currentConversation = updated
        }
    }

    public func send(text: String, providerId: String) async {
        guard let convoId = currentConversation?.id else {
            await createNew()
            guard let newId = currentConversation?.id else { return }
            await actuallySend(text: text, providerId: providerId, conversationId: newId)
            return
        }
        await actuallySend(text: text, providerId: providerId, conversationId: convoId)
    }

    private func actuallySend(text: String, providerId: String, conversationId: String) async {
        error = nil
        sending = true
        defer { sending = false }
        let optimisticUser = Message(
            conversationId: conversationId,
            role: .user,
            content: text
        )
        messages.append(optimisticUser)
        do {
            let assistant = try await services.chat.send(
                conversationId: conversationId,
                providerId: providerId,
                userText: text
            )
            messages = (try? await services.conversations.messages(for: conversationId)) ?? messages
            _ = assistant
            if let updated = try? await services.conversations.get(conversationId) {
                currentConversation = updated
                if let idx = conversations.firstIndex(where: { $0.id == conversationId }) {
                    conversations[idx] = updated
                }
            }
            Sounds.play(.chime)
        } catch let chatError as ChatRequestError {
            error = chatError.localizedDescription
            messages.removeAll(where: { $0.id == optimisticUser.id })
            Sounds.play(.error)
        } catch {
            self.error = error.localizedDescription
            messages.removeAll(where: { $0.id == optimisticUser.id })
            Sounds.play(.error)
        }
    }
}
