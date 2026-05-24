import Foundation

public struct Services: Sendable {
    public let database: AppDatabase
    public let sessions: SessionService
    public let tasks: TaskService
    public let providerConfigs: ProviderConfigService
    public let settings: SettingsService
    public let ai: AIService
    public let speech: SpeechService
    public let memory: MemoryService
    public let activity: ActivityService
    public let canvases: CanvasService
    public let search: SearchService

    public init(database: AppDatabase) {
        self.database = database
        let sessions = SessionService(database: database)
        let tasks = TaskService(database: database)
        let providerConfigs = ProviderConfigService(database: database)
        let settings = SettingsService(database: database)
        let memory = MemoryService(settings: settings)
        let activity = ActivityService(database: database)
        self.sessions = sessions
        self.tasks = tasks
        self.providerConfigs = providerConfigs
        self.settings = settings
        self.memory = memory
        self.activity = activity
        self.canvases = CanvasService(database: database)
        self.search = SearchService(database: database)
        self.ai = AIService(
            database: database,
            providerConfigs: providerConfigs,
            sessions: sessions,
            tasks: tasks,
            memory: memory,
            activity: activity
        )
        self.speech = SpeechService(providerConfigs: providerConfigs, settings: settings)
    }

    public static func makeOnDisk() async throws -> Services {
        let db = try AppDatabase.makeOnDisk(at: AppDatabase.defaultDatabaseURL)
        return Services(database: db)
    }
}
