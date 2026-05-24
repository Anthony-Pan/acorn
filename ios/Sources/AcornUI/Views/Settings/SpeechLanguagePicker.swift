import SwiftUI
import AcornCore
#if canImport(Speech)
import Speech
#endif

public struct SpeechLanguagePicker: View {
    @Environment(SettingsStore.self) private var settings
    @Environment(\.dismiss) private var dismiss
    @State private var searchText: String = ""

    public init() {}

    public var body: some View {
        List {
            Section {
                Button(action: chooseAutoDetect) {
                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Auto (match system)")
                                .font(.acornBody.weight(.semibold))
                            Text(autoSummary)
                                .font(.acornCaption)
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                        if settings.speechLanguage == nil {
                            Image(systemName: "checkmark.circle.fill")
                                .foregroundStyle(Color.acornAccent)
                        }
                    }
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            } footer: {
                Text("If unset, voice transcription falls back to your iPhone language.")
                    .font(.acornCaption)
            }

            ForEach(filteredGroupedLocales, id: \.0) { region, locales in
                Section(region) {
                    ForEach(locales, id: \.identifier) { locale in
                        Button(action: { choose(locale) }) {
                            HStack {
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(Self.displayName(for: locale))
                                        .font(.acornBody)
                                    Text(locale.identifier)
                                        .font(.acornCaption.monospaced())
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                                if settings.speechLanguage == locale.identifier {
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundStyle(Color.acornAccent)
                                }
                            }
                            .contentShape(Rectangle())
                        }
                        .buttonStyle(.plain)
                    }
                }
            }
        }
        .searchable(text: $searchText, prompt: "Search 70+ languages")
        .navigationTitle("Voice language")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }

    private var autoSummary: String {
        let id = Locale.preferredLanguages.first ?? Locale.current.identifier
        guard let locale = supportedLocaleMatch(for: id) else {
            return "\(id) (system)"
        }
        return Self.displayName(for: locale)
    }

    private func supportedLocaleMatch(for identifier: String) -> Locale? {
        let lower = identifier.lowercased()
        return allLocales.first {
            $0.identifier.lowercased() == lower || lower.hasPrefix($0.identifier.lowercased().prefix(2))
        }
    }

    private var allLocales: [Locale] {
        #if canImport(Speech)
        SFSpeechRecognizer.supportedLocales()
            .sorted(by: { Self.displayName(for: $0) < Self.displayName(for: $1) })
        #else
        []
        #endif
    }

    private var filteredGroupedLocales: [(String, [Locale])] {
        let pool = allLocales.filter { locale in
            guard !searchText.isEmpty else { return true }
            let needle = searchText.lowercased()
            return Self.displayName(for: locale).lowercased().contains(needle)
                || locale.identifier.lowercased().contains(needle)
        }
        let grouped = Dictionary(grouping: pool) { (locale) -> String in
            #if os(iOS)
            if let region = locale.region?.identifier {
                return Locale.current.localizedString(forRegionCode: region) ?? region
            }
            #endif
            return "Other"
        }
        return grouped.sorted(by: { $0.key < $1.key })
    }

    private func chooseAutoDetect() {
        _Concurrency.Task { await settings.setSpeechLanguage(nil); dismiss() }
    }

    private func choose(_ locale: Locale) {
        _Concurrency.Task { await settings.setSpeechLanguage(locale.identifier); dismiss() }
    }

    static func displayName(for locale: Locale) -> String {
        let inUserLanguage = Locale.current.localizedString(forIdentifier: locale.identifier)
        let inNative = locale.localizedString(forIdentifier: locale.identifier)
        if let n = inNative, let u = inUserLanguage, n.caseInsensitiveCompare(u) != .orderedSame {
            return "\(u) — \(n)"
        }
        return inUserLanguage ?? locale.identifier
    }
}
