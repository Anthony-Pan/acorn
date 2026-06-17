import Foundation
import AppIntents

@available(iOS 17.0, macOS 14.0, *)
public struct StashIntent: AppIntent {
    public static let title: LocalizedStringResource = "Stash to Acorn"

    public static let description = IntentDescription(
        "Capture a messy thought and let Acorn break it into focused task cards."
    )

    public static let openAppWhenRun: Bool = false

    @Parameter(
        title: "What's on your mind?",
        description: "Free-form text — Acorn will decompose it into tasks.",
        inputOptions: .init(
            keyboardType: .default,
            capitalizationType: .sentences,
            multiline: true
        )
    )
    public var text: String

    public init() {}

    public init(text: String) {
        self.text = text
    }

    @MainActor
    public func perform() async throws -> some IntentResult & ProvidesDialog {
        let services = try await AcornServices.shared.current()
        let language = Locale.current.identifier
        let activeProviderId =
            (try? await services.settings.get(.activeProvider))
            ?? ProviderCatalog.defaultId

        let stream = await services.ai.stash(
            rawInput: text,
            language: language,
            providerId: activeProviderId
        )

        var taskCount = 0
        var summary = ""
        for try await event in stream {
            switch event {
            case .task: taskCount += 1
            case .summary(let s): summary = s
            case .done, .progress, .sessionCreated: break
            }
        }

        let countLabel = taskCount == 1 ? "1 task" : "\(taskCount) tasks"
        let dialog = summary.isEmpty
            ? "Stashed \(countLabel)."
            : "Stashed \(countLabel). \(summary)"
        return .result(dialog: IntentDialog(stringLiteral: dialog))
    }
}

@available(iOS 17.0, macOS 14.0, *)
public struct OpenAcornIntent: AppIntent {
    public static let title: LocalizedStringResource = "Open Acorn"
    public static let description = IntentDescription("Bring Acorn to the foreground.")
    public static let openAppWhenRun: Bool = true

    public init() {}

    @MainActor
    public func perform() async throws -> some IntentResult {
        .result()
    }
}
