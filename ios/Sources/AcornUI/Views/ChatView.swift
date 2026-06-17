import SwiftUI
import AcornCore

public struct ChatView: View {
    @Environment(AppRouter.self) private var router
    @Environment(SettingsStore.self) private var settings
    @Environment(ProvidersStore.self) private var providers
    let services: Services

    @State private var store: ChatStore
    @State private var draft: String = ""
    @State private var showSidebar = true

    public init(services: Services) {
        self.services = services
        _store = State(initialValue: ChatStore(services: services))
    }

    public var body: some View {
        NavigationStack {
            HStack(spacing: 0) {
                if showSidebar {
                    sidebar
                        .frame(width: 220)
                    Divider()
                }
                main
            }
            .navigationTitle(store.currentConversation?.title ?? "Chat")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        showSidebar.toggle()
                    } label: {
                        Image(systemName: "sidebar.left")
                    }
                }
                ToolbarItem(placement: .primaryAction) {
                    Menu {
                        Button {
                            _Concurrency.Task { await store.createNew() }
                        } label: {
                            Label("New chat", systemImage: "plus")
                        }
                        Button(role: .destructive) {
                            _Concurrency.Task { await store.deleteCurrent() }
                        } label: {
                            Label("Delete this chat", systemImage: "trash")
                        }
                        .disabled(store.currentConversation == nil)
                        Divider()
                        Button {
                            router.navigate(.input)
                        } label: {
                            Label("Back to stash", systemImage: "tray.and.arrow.up")
                        }
                    } label: {
                        Image(systemName: "ellipsis.circle")
                    }
                }
            }
            .background(Color.warmCream.opacity(0.4).ignoresSafeArea())
        }
        .task {
            await store.hydrate()
        }
    }

    private var sidebar: some View {
        VStack(spacing: 0) {
            Button {
                _Concurrency.Task { await store.createNew() }
            } label: {
                Label("New chat", systemImage: "plus.circle.fill")
                    .font(.acornButton)
                    .foregroundStyle(Color.acornAccent)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 12)
            }
            .buttonStyle(.plain)
            Divider()
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 4) {
                    ForEach(store.conversations) { convo in
                        Button {
                            _Concurrency.Task { await store.select(convo.id) }
                        } label: {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(convo.title)
                                    .font(.acornCaption.weight(.semibold))
                                    .lineLimit(1)
                                Text(convo.lastMessageAt, style: .relative)
                                    .font(.acornCaption)
                                    .foregroundStyle(.secondary)
                            }
                            .padding(8)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(
                                convo.id == store.currentConversation?.id
                                    ? Color.acornAccent.opacity(0.18)
                                    : Color.clear
                            )
                            .clipShape(RoundedRectangle(cornerRadius: 8))
                        }
                        .buttonStyle(.plain)
                    }
                    if store.conversations.isEmpty {
                        Text("No chats yet")
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

    private var main: some View {
        VStack(spacing: 0) {
            messageList
            inputBar
        }
    }

    private var messageList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 12) {
                    ForEach(store.messages) { msg in
                        MessageBubble(message: msg)
                            .id(msg.id)
                    }
                    if store.sending {
                        HStack {
                            ProgressView().scaleEffect(0.7).tint(Color.acornAccent)
                            Text("Thinking…")
                                .font(.acornCaption)
                                .foregroundStyle(.secondary)
                            Spacer()
                        }
                        .padding(.horizontal, 16)
                    }
                    if let err = store.error {
                        Text(err)
                            .font(.acornCaption)
                            .foregroundStyle(Color.dustyRose)
                            .padding(.horizontal, 16)
                    }
                }
                .padding(.vertical, 16)
            }
            .onChange(of: store.messages.count) { _, _ in
                if let last = store.messages.last {
                    withAnimation { proxy.scrollTo(last.id, anchor: .bottom) }
                }
            }
        }
    }

    private var inputBar: some View {
        HStack(spacing: 8) {
            TextField("Ask Acorn…", text: $draft, axis: .vertical)
                .font(.acornBody)
                .lineLimit(1...5)
                .padding(10)
                .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 14))
            Button {
                send()
            } label: {
                Image(systemName: "paperplane.fill")
                    .font(.title3)
                    .foregroundStyle(.white)
                    .frame(width: 40, height: 40)
                    .background(Color.acornAccent.gradient)
                    .clipShape(Circle())
                    .shadow(color: Color.acornAccent.opacity(0.4), radius: 6, x: 0, y: 3)
            }
            .buttonStyle(.plain)
            .disabled(draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.sending)
        }
        .padding(12)
    }

    private func send() {
        let text = draft
        draft = ""
        _Concurrency.Task {
            await store.send(text: text, providerId: settings.activeProviderId)
        }
    }
}

struct MessageBubble: View {
    let message: Message

    var body: some View {
        HStack(alignment: .top, spacing: 0) {
            if message.role == .user { Spacer(minLength: 40) }
            VStack(alignment: alignment, spacing: 4) {
                renderedContent
                    .font(.acornBody)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)
                    .background(bubbleBackground)
                    .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
                    .foregroundStyle(bubbleForeground)
            }
            if message.role == .assistant { Spacer(minLength: 40) }
        }
        .padding(.horizontal, 12)
    }

    private var alignment: HorizontalAlignment {
        message.role == .user ? .trailing : .leading
    }

    private var bubbleBackground: AnyShapeStyle {
        switch message.role {
        case .user: AnyShapeStyle(Color.acornAccent.gradient)
        case .assistant: AnyShapeStyle(.regularMaterial)
        case .system: AnyShapeStyle(Color.warmCream.opacity(0.4))
        case .tool: AnyShapeStyle(Color.mossGreen.opacity(0.18))
        }
    }

    private var bubbleForeground: Color {
        message.role == .user ? .white : .primary
    }

    private var renderedContent: some View {
        let attributed = (try? AttributedString(
            markdown: message.content,
            options: .init(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        )) ?? AttributedString(message.content)
        return Text(attributed)
    }
}
