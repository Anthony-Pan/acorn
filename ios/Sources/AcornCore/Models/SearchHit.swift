import Foundation

public struct SearchHit: Codable, Sendable, Identifiable {
    public let source: String
    public let sourceId: String
    public let title: String
    public let snippet: String
    public let createdAt: Date
    public let conversationId: String?

    public var id: String { "\(source):\(sourceId)" }

    public init(
        source: String,
        sourceId: String,
        title: String,
        snippet: String,
        createdAt: Date,
        conversationId: String? = nil
    ) {
        self.source = source
        self.sourceId = sourceId
        self.title = title
        self.snippet = snippet
        self.createdAt = createdAt
        self.conversationId = conversationId
    }
}
