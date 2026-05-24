import SwiftUI
import AcornCore

public struct CanvasView: View {
    @Environment(AppRouter.self) private var router
    let services: Services

    @State private var canvases: [AcornCore.Canvas] = []
    @State private var selectedId: String?
    @State private var title: String = ""
    @State private var content: String = ""
    @State private var loading = true
    @State private var preview = false
    @State private var saveTask: _Concurrency.Task<Void, Never>?

    public init(services: Services) {
        self.services = services
    }

    public var body: some View {
        NavigationStack {
            HStack(spacing: 0) {
                sidebar
                    .frame(maxWidth: 200)
                Divider()
                editor
                    .frame(maxWidth: .infinity)
            }
            .navigationTitle("Notebook")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { router.navigate(.input) }
                        .font(.acornButton)
                }
                ToolbarItem(placement: .primaryAction) {
                    Menu {
                        Button {
                            preview.toggle()
                        } label: {
                            Label(preview ? "Edit" : "Preview", systemImage: preview ? "pencil" : "eye")
                        }
                        Button(role: .destructive) {
                            deleteCurrent()
                        } label: {
                            Label("Delete canvas", systemImage: "trash")
                        }
                        .disabled(selectedId == nil)
                    } label: {
                        Image(systemName: "ellipsis.circle")
                    }
                }
            }
            .background(Color.warmCream.opacity(0.4).ignoresSafeArea())
        }
        .task {
            await loadAll()
        }
    }

    private var sidebar: some View {
        VStack(spacing: 0) {
            Button(action: createNew) {
                Label("New canvas", systemImage: "plus.circle.fill")
                    .font(.acornButton)
                    .foregroundStyle(Color.acornAccent)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 12)
            }
            .buttonStyle(.plain)
            Divider()
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 4) {
                    ForEach(canvases) { canvas in
                        Button {
                            select(canvas)
                        } label: {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(canvas.title)
                                    .font(.acornCaption.weight(.semibold))
                                    .lineLimit(1)
                                Text(canvas.updatedAt, style: .relative)
                                    .font(.acornCaption)
                                    .foregroundStyle(.secondary)
                            }
                            .padding(8)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(
                                canvas.id == selectedId
                                    ? Color.acornAccent.opacity(0.18)
                                    : Color.clear
                            )
                            .clipShape(RoundedRectangle(cornerRadius: 8))
                        }
                        .buttonStyle(.plain)
                    }
                    if canvases.isEmpty && !loading {
                        Text("No canvases yet")
                            .font(.acornCaption)
                            .foregroundStyle(.secondary)
                            .padding()
                    }
                }
                .padding(8)
            }
        }
        .background(.regularMaterial)
    }

    @ViewBuilder
    private var editor: some View {
        if selectedId == nil && !loading {
            VStack(spacing: 12) {
                Text("📝").font(.system(size: 56))
                Text("Pick a canvas or create one")
                    .font(.acornHeader)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if loading {
            ProgressView().controlSize(.large).tint(Color.acornAccent)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            VStack(alignment: .leading, spacing: 0) {
                TextField("Title", text: $title)
                    .font(.acornHeader)
                    .textFieldStyle(.plain)
                    .padding(.horizontal, 16)
                    .padding(.top, 16)
                    .onChange(of: title) { _, _ in scheduleSave() }
                Divider().padding(.vertical, 8)
                if preview {
                    ScrollView {
                        renderedContent
                            .padding(16)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } else {
                    TextEditor(text: $content)
                        .font(.acornBody)
                        .scrollContentBackground(.hidden)
                        .padding(.horizontal, 12)
                        .onChange(of: content) { _, _ in scheduleSave() }
                }
            }
        }
    }

    private var renderedContent: some View {
        let attributed = (try? AttributedString(
            markdown: content,
            options: .init(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        )) ?? AttributedString(content)
        return Text(attributed)
            .font(.acornBody)
    }

    private func loadAll() async {
        loading = true
        defer { loading = false }
        canvases = (try? await services.canvases.list()) ?? []
        if let first = canvases.first {
            await selectAsync(first)
        }
    }

    private func select(_ canvas: AcornCore.Canvas) {
        _Concurrency.Task { await selectAsync(canvas) }
    }

    private func selectAsync(_ canvas: AcornCore.Canvas) async {
        flushPendingSave()
        selectedId = canvas.id
        title = canvas.title
        content = canvas.content
        try? await services.activity.record(kind: .canvasEdit, content: canvas.title)
    }

    private func createNew() {
        _Concurrency.Task {
            flushPendingSave()
            if let created = try? await services.canvases.create(title: nil) {
                canvases.insert(created, at: 0)
                await selectAsync(created)
            }
        }
    }

    private func deleteCurrent() {
        guard let id = selectedId else { return }
        _Concurrency.Task {
            try? await services.canvases.delete(id)
            canvases.removeAll(where: { $0.id == id })
            selectedId = canvases.first?.id
            if let first = canvases.first {
                title = first.title
                content = first.content
            } else {
                title = ""
                content = ""
            }
        }
    }

    private func scheduleSave() {
        saveTask?.cancel()
        saveTask = _Concurrency.Task {
            try? await _Concurrency.Task.sleep(for: .milliseconds(800))
            guard !_Concurrency.Task.isCancelled, let id = selectedId else { return }
            if let updated = try? await services.canvases.update(id: id, title: title, content: content) {
                if let idx = canvases.firstIndex(where: { $0.id == id }) {
                    canvases[idx] = updated
                }
            }
        }
    }

    private func flushPendingSave() {
        saveTask?.cancel()
        guard let id = selectedId, !title.isEmpty || !content.isEmpty else { return }
        _Concurrency.Task.detached(priority: .userInitiated) {
            _ = try? await services.canvases.update(id: id, title: title, content: content)
        }
    }
}
