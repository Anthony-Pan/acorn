import Foundation
import GRDB

public actor AIService {
    private let database: AppDatabase
    private let providerConfigs: ProviderConfigService
    private let sessions: SessionService
    private let tasks: TaskService
    private let memory: MemoryService
    private let activity: ActivityService

    public init(
        database: AppDatabase,
        providerConfigs: ProviderConfigService,
        sessions: SessionService,
        tasks: TaskService,
        memory: MemoryService,
        activity: ActivityService
    ) {
        self.database = database
        self.providerConfigs = providerConfigs
        self.sessions = sessions
        self.tasks = tasks
        self.memory = memory
        self.activity = activity
    }

    public nonisolated func listProviders() -> [ProviderMetadata] {
        ProviderCatalog.all
    }

    public func makeProvider(id providerId: String) async throws -> any Provider {
        guard let metadata = ProviderCatalog.get(providerId) else {
            throw ProviderError.notImplemented("Unknown provider id: \(providerId)")
        }
        let config = try await providerConfigs.get(providerId)
        var apiKey: String?
        if metadata.requiresApiKey {
            apiKey = try await providerConfigs.readCredentials(providerId: providerId)
            if apiKey == nil || apiKey?.isEmpty == true {
                throw ProviderError.missingCredentials(providerId)
            }
        }
        return try ProviderFactory.build(ProviderInputs(
            metadata: metadata,
            apiKey: apiKey,
            model: config?.selectedModel,
            customEndpoint: config?.customEndpoint
        ))
    }

    public func decompose(
        providerId: String,
        request: DecomposeRequest
    ) -> AsyncThrowingStream<DecomposeEvent, Error> {
        AsyncThrowingStream { continuation in
            _Concurrency.Task {
                do {
                    let provider = try await makeProvider(id: providerId)
                    try await providerConfigs.touch(providerId)
                    let stream = provider.decompose(request)
                    for try await event in stream {
                        continuation.yield(event)
                    }
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }

    public func stash(
        rawInput: String,
        language: String,
        providerId: String
    ) -> AsyncThrowingStream<StashEvent, Error> {
        AsyncThrowingStream { continuation in
            _Concurrency.Task {
                do {
                    let session = try await sessions.create(rawInput: rawInput, language: language)
                    continuation.yield(.sessionCreated(session))
                    try? await activity.record(kind: .stash, content: rawInput)

                    let sharedMemory = (try? await memory.read()) ?? ""
                    let request = DecomposeRequest(
                        rawInput: rawInput,
                        language: language,
                        userContext: sharedMemory.isEmpty ? nil : sharedMemory
                    )
                    let stream = decompose(providerId: providerId, request: request)

                    var orderCounter: Int64 = 0
                    var capturedSummary = ""
                    for try await event in stream {
                        switch event {
                        case .progress:
                            continuation.yield(.progress)
                        case .task(let decomposed):
                            let stored = try await tasks.insert(
                                sessionId: session.id,
                                title: decomposed.title,
                                description: decomposed.description,
                                durationMinutes: Int64(decomposed.durationMinutes),
                                priority: decomposed.priority,
                                orderIndex: orderCounter,
                                subtasks: decomposed.subtasks
                            )
                            orderCounter += 1
                            continuation.yield(.task(stored))
                        case .summary(let s):
                            capturedSummary = s
                            continuation.yield(.summary(s))
                        case .done:
                            let metadata = ProviderCatalog.get(providerId)
                            let model = metadata?.defaultModel ?? "auto"
                            _ = try await sessions.finalize(
                                sessionId: session.id,
                                aiSummary: capturedSummary,
                                providerId: providerId,
                                model: model
                            )
                            continuation.yield(.done)
                        }
                    }
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }
}

public enum StashEvent: Sendable {
    case sessionCreated(Session)
    case progress
    case task(Task)
    case summary(String)
    case done
}
