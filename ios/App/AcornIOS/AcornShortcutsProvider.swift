import AppIntents
import AcornCore

@available(iOS 17.0, *)
struct AcornShortcutsProvider: AppShortcutsProvider {
    static var appShortcuts: [AppShortcut] {
        AppShortcut(
            intent: StashIntent(),
            phrases: [
                "Stash to \(.applicationName)",
                "Save to \(.applicationName): \(\.$text)",
                "\(.applicationName), stash this: \(\.$text)",
            ],
            shortTitle: "Stash to Acorn",
            systemImageName: "leaf.fill"
        )
        AppShortcut(
            intent: OpenAcornIntent(),
            phrases: [
                "Open \(.applicationName)",
            ],
            shortTitle: "Open Acorn",
            systemImageName: "leaf"
        )
    }
}
