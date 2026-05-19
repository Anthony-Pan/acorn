import AppIntents
import Foundation

public struct StashIntent: AppIntent {
    public static var title: LocalizedStringResource = "Stash a thought into Acorn"
    public static var description = IntentDescription(
        "Drop any text into Acorn and let it break the thought down into focused task cards."
    )
    public static var openAppWhenRun: Bool = true

    @Parameter(title: "What's on your mind", inputOptions: .init(multiline: true))
    public var text: String

    public init() {}

    public init(text: String) {
        self.text = text
    }

    public func perform() async throws -> some IntentResult {
        let encoded =
            text.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? ""
        guard let url = URL(string: "acorn://stash?text=\(encoded)") else {
            throw IntentError.invalidInput
        }
        await AcornDeepLinkLauncher.open(url)
        return .result()
    }
}

public struct SummonIntent: AppIntent {
    public static var title: LocalizedStringResource = "Summon Acorn"
    public static var description = IntentDescription(
        "Pop up the Acorn quick window so you can talk or type your stash."
    )
    public static var openAppWhenRun: Bool = true

    public init() {}

    public func perform() async throws -> some IntentResult {
        guard let url = URL(string: "acorn://summon") else {
            throw IntentError.invalidInput
        }
        await AcornDeepLinkLauncher.open(url)
        return .result()
    }
}

public struct OpenAcornIntent: AppIntent {
    public static var title: LocalizedStringResource = "Open Acorn"
    public static var description = IntentDescription("Bring the Acorn main window to the front.")
    public static var openAppWhenRun: Bool = true

    public init() {}

    public func perform() async throws -> some IntentResult {
        guard let url = URL(string: "acorn://show") else {
            throw IntentError.invalidInput
        }
        await AcornDeepLinkLauncher.open(url)
        return .result()
    }
}

public struct AcornShortcuts: AppShortcutsProvider {
    public static var appShortcuts: [AppShortcut] {
        AppShortcut(
            intent: StashIntent(),
            phrases: [
                "Stash with \(.applicationName)",
                "Stash this in \(.applicationName)",
                "Drop into \(.applicationName)"
            ],
            shortTitle: "Stash",
            systemImageName: "leaf.fill"
        )

        AppShortcut(
            intent: SummonIntent(),
            phrases: [
                "Summon \(.applicationName)",
                "Open \(.applicationName) quick window"
            ],
            shortTitle: "Summon",
            systemImageName: "command"
        )

        AppShortcut(
            intent: OpenAcornIntent(),
            phrases: [
                "Open \(.applicationName)",
                "Show \(.applicationName)"
            ],
            shortTitle: "Open",
            systemImageName: "macwindow"
        )
    }
}

public enum IntentError: Swift.Error, CustomLocalizedStringResourceConvertible {
    case invalidInput

    public var localizedStringResource: LocalizedStringResource {
        switch self {
        case .invalidInput:
            return "Acorn could not build a valid acorn:// URL from this input."
        }
    }
}

enum AcornDeepLinkLauncher {
    static func open(_ url: URL) async {
        #if canImport(AppKit)
        await MainActor.run {
            NSWorkspace.shared.open(url)
        }
        #endif
    }
}

#if canImport(AppKit)
import AppKit
#endif
