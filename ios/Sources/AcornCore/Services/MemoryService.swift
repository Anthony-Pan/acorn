import Foundation

public actor MemoryService {
    private let settings: SettingsService

    public init(settings: SettingsService) {
        self.settings = settings
    }

    public func read() async throws -> String {
        (try await settings.get(.sharedMemory)) ?? ""
    }

    public func write(_ value: String) async throws {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty {
            try await settings.remove(.sharedMemory)
        } else {
            try await settings.set(.sharedMemory, value: trimmed)
        }
    }

    public func clear() async throws {
        try await settings.remove(.sharedMemory)
    }
}
