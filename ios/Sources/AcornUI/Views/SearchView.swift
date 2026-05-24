import SwiftUI
import AcornCore

public struct SearchView: View {
    @Environment(AppRouter.self) private var router
    let services: Services

    @State private var query: String = ""
    @State private var hits: [SearchHit] = []
    @State private var searching = false
    @State private var debounceTask: _Concurrency.Task<Void, Never>?

    public init(services: Services) {
        self.services = services
    }

    public var body: some View {
        NavigationStack {
            List {
                if hits.isEmpty && !query.isEmpty && !searching {
                    Text("No matches for \"\(query)\".")
                        .font(.acornBody)
                        .foregroundStyle(.secondary)
                } else if hits.isEmpty && query.isEmpty {
                    Text("Search across your sessions, tasks, and chat messages. All local — nothing leaves your phone.")
                        .font(.acornCaption)
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(groupedHits, id: \.0) { source, items in
                        Section(sourceLabel(source)) {
                            ForEach(items) { hit in
                                SearchHitRow(hit: hit)
                            }
                        }
                    }
                }
            }
            #if os(iOS)
            .listStyle(.insetGrouped)
            #endif
            .scrollContentBackground(.hidden)
            .navigationTitle("Search")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .searchable(text: $query, prompt: "find anything")
            .onChange(of: query) { _, newValue in
                schedule(query: newValue)
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { router.navigate(.input) }
                        .font(.acornButton)
                }
            }
            .background(Color.warmCream.opacity(0.4).ignoresSafeArea())
        }
    }

    private var groupedHits: [(String, [SearchHit])] {
        let groups = Dictionary(grouping: hits) { $0.source }
        return groups.sorted(by: { $0.key < $1.key })
    }

    private func sourceLabel(_ source: String) -> String {
        switch source {
        case "session": "Stashes"
        case "task": "Tasks"
        case "message": "Chat messages"
        default: source.capitalized
        }
    }

    private func schedule(query newValue: String) {
        debounceTask?.cancel()
        let value = newValue.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !value.isEmpty else {
            hits = []
            searching = false
            return
        }
        debounceTask = _Concurrency.Task {
            try? await _Concurrency.Task.sleep(for: .milliseconds(250))
            guard !_Concurrency.Task.isCancelled else { return }
            searching = true
            hits = (try? await services.search.query(value, limit: 60)) ?? []
            searching = false
            try? await services.activity.record(kind: .search, content: value)
        }
    }
}

struct SearchHitRow: View {
    let hit: SearchHit

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(hit.title)
                .font(.acornBody.weight(.semibold))
                .lineLimit(1)
            renderedSnippet
                .font(.acornCaption)
                .foregroundStyle(.secondary)
                .lineLimit(3)
            Text(hit.createdAt, style: .relative)
                .font(.acornCaption)
                .foregroundStyle(.tertiary)
        }
        .padding(.vertical, 2)
    }

    private var renderedSnippet: Text {
        var attributed = AttributedString(hit.snippet)
        let raw = hit.snippet
        var cursor = raw.startIndex
        while let openRange = raw.range(of: "<<", range: cursor..<raw.endIndex) {
            cursor = openRange.upperBound
            if let closeRange = raw.range(of: ">>", range: cursor..<raw.endIndex) {
                let inner = String(raw[openRange.upperBound..<closeRange.lowerBound])
                if let attrRange = attributed.range(of: inner) {
                    attributed[attrRange].font = .system(.caption, design: .rounded).bold()
                    attributed[attrRange].foregroundColor = Color.acornAccent
                }
                cursor = closeRange.upperBound
            } else {
                break
            }
        }
        let cleaned = attributed.characters
            .reduce(into: AttributedString("")) { acc, ch in acc.append(AttributedString(String(ch))) }
        _ = cleaned
        // Strip << >> markers from displayed text by replacing them with empty AttributedStrings.
        var display = attributed
        for marker in ["<<", ">>"] {
            while let r = display.range(of: marker) {
                display.replaceSubrange(r, with: AttributedString(""))
            }
        }
        return Text(display)
    }
}
