import Foundation

public actor AcornServices {
    public static let shared = AcornServices()

    private var services: Services?

    private init() {}

    public func current() async throws -> Services {
        if let services { return services }
        let new = try await Services.makeOnDisk()
        services = new
        return new
    }
}
