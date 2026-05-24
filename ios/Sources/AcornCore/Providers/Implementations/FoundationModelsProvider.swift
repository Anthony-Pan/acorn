import Foundation

#if canImport(FoundationModels)
import FoundationModels

@available(iOS 26.0, macOS 26.0, visionOS 26.0, *)
public struct FoundationModelsProvider: Provider {
    public let metadata: ProviderMetadata
    public let supportsTools: Bool = false

    public init(metadata: ProviderMetadata) {
        self.metadata = metadata
    }

    public func chat(turns: [ChatTurn], systemPrompt: String, temperature: Double) async throws -> String {
        try await validateCredentials()
        let session = LanguageModelSession(instructions: Instructions(systemPrompt))
        let userPrompt = turns.last(where: { $0.role == "user" })?.content ?? ""
        let response = try await session.respond(
            to: Prompt(userPrompt),
            options: GenerationOptions(temperature: temperature)
        )
        return response.content
    }

    public func validateCredentials() async throws {
        switch SystemLanguageModel.default.availability {
        case .available:
            return
        case .unavailable(.deviceNotEligible):
            throw ProviderError.modelUnavailable(
                reason: "This device doesn't support Apple Intelligence."
            )
        case .unavailable(.appleIntelligenceNotEnabled):
            throw ProviderError.modelUnavailable(
                reason: "Turn on Apple Intelligence in Settings → Apple Intelligence & Siri."
            )
        case .unavailable(.modelNotReady):
            throw ProviderError.modelUnavailable(
                reason: "The on-device model is still downloading. Try again in a minute."
            )
        case .unavailable:
            throw ProviderError.modelUnavailable(reason: "Apple Intelligence is unavailable.")
        @unknown default:
            throw ProviderError.modelUnavailable(reason: "Unknown Apple Intelligence state.")
        }
    }

    public func decompose(_ request: DecomposeRequest) -> AsyncThrowingStream<DecomposeEvent, Error> {
        AsyncThrowingStream { continuation in
            _Concurrency.Task {
                do {
                    try await validateCredentials()
                    let session = LanguageModelSession(
                        instructions: Instructions(
                            DecomposePrompts.systemPrompt(language: request.language, sharedMemory: request.userContext)
                        )
                    )
                    let stream = session.streamResponse(
                        to: Prompt(DecomposePrompts.userPrompt(request)),
                        generating: GenerableDecomposeResponse.self,
                        options: GenerationOptions(temperature: 0.3)
                    )

                    var emittedTaskCount = 0
                    for try await snapshot in stream {
                        let partialTasks = snapshot.content.tasks ?? []
                        while emittedTaskCount < partialTasks.count {
                            let partial = partialTasks[emittedTaskCount]
                            guard
                                let title = partial.title,
                                !title.isEmpty,
                                let priority = partial.priority,
                                let duration = partial.durationMinutes
                            else {
                                break
                            }
                            let task = DecomposedTask(
                                title: title,
                                description: partial.description,
                                durationMinutes: max(1, min(duration, 480)),
                                priority: priority.toCorePriority,
                                order: emittedTaskCount,
                                subtasks: (partial.subtasks ?? []).compactMap { p -> DecomposedSubtask? in
                                    guard let t = p.title, !t.isEmpty else { return nil }
                                    return DecomposedSubtask(title: t, done: p.done ?? false)
                                }
                            )
                            continuation.yield(.task(task))
                            try await _Concurrency.Task.sleep(for: DecomposeEmitter.popInGap)
                            emittedTaskCount += 1
                        }
                    }

                    let finalResponse = try await stream.collect()
                    continuation.yield(.summary(finalResponse.content.summary))
                    continuation.yield(.done)
                    continuation.finish()
                } catch LanguageModelSession.GenerationError.refusal(let refusal, _) {
                    let explanation = (try? await refusal.explanation)?.content ?? "Model refused."
                    continuation.finish(throwing: ProviderError.providerResponse(explanation))
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }
}

@available(iOS 26.0, macOS 26.0, visionOS 26.0, *)
@Generable
struct GenerableDecomposeResponse {
    @Guide(description: "Ordered list of focused task cards extracted from the dump.")
    var tasks: [GenerableTask]

    @Guide(description: "One short sentence summary, no more than 140 chars.")
    var summary: String
}

@available(iOS 26.0, macOS 26.0, visionOS 26.0, *)
@Generable
struct GenerableTask {
    @Guide(description: "Short task title, 5 to 15 words, verb-first, no trailing punctuation.")
    var title: String

    @Guide(description: "Optional one-sentence detail. Omit if title is self-evident.")
    var description: String?

    @Guide(description: "Estimated minutes to complete (1 to 480).")
    var durationMinutes: Int

    @Guide(description: "Urgency: high / medium / low.")
    var priority: GenerablePriority

    @Guide(description: "Zero-indexed execution order within the day.")
    var order: Int

    @Guide(description: "Optional nested checklist items. Usually empty.")
    var subtasks: [GenerableSubtask]
}

@available(iOS 26.0, macOS 26.0, visionOS 26.0, *)
@Generable
struct GenerableSubtask {
    @Guide(description: "One concrete step within the parent task.")
    var title: String

    var done: Bool?
}

@available(iOS 26.0, macOS 26.0, visionOS 26.0, *)
@Generable
enum GenerablePriority: String, Codable {
    case high
    case medium
    case low

    var toCorePriority: Priority {
        switch self {
        case .high: .high
        case .medium: .medium
        case .low: .low
        }
    }
}

#endif
