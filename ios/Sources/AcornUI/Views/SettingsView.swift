import SwiftUI
import AcornCore

public struct SettingsView: View {
    @Environment(SettingsStore.self) private var settings
    @Environment(ProvidersStore.self) private var providers
    @Environment(AppRouter.self) private var router

    public init() {}

    public var body: some View {
        NavigationStack {
            List {
                Section("Active provider") {
                    ForEach(providers.catalog.filter { $0.status == .available || $0.status == .platformUnsupported }) { metadata in
                        ProviderRow(
                            metadata: metadata,
                            isActive: settings.activeProviderId == metadata.id,
                            hasCredentials: providers.hasCredentials[metadata.id] ?? !metadata.requiresApiKey,
                            requiresKey: metadata.requiresApiKey,
                            unsupported: metadata.status == .platformUnsupported,
                            select: { _Concurrency.Task { await settings.setActiveProvider(metadata.id) } }
                        )
                    }
                }

                Section {
                    if let activeMetadata = providers.metadata(for: settings.activeProviderId),
                       activeMetadata.requiresApiKey {
                        NavigationLink {
                            CredentialsView(metadata: activeMetadata)
                        } label: {
                            Label(
                                providers.hasCredentials[activeMetadata.id] == true
                                    ? "Update API key for \(activeMetadata.displayName)"
                                    : "Set API key for \(activeMetadata.displayName)",
                                systemImage: "key.fill"
                            )
                        }
                    }
                }

                Section("Acorn iOS") {
                    LabeledContent("Version") {
                        Text(AcornVersion.current).foregroundStyle(.secondary)
                    }
                    LabeledContent("macOS parity") {
                        Text(AcornVersion.macOSEquivalent).foregroundStyle(.secondary)
                    }
                }

                if let err = providers.lastError {
                    Section {
                        Text(err)
                            .font(.acornCaption)
                            .foregroundStyle(Color.dustyRose)
                    }
                }
            }
            .navigationTitle("Settings")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") {
                        router.navigate(.input)
                    }
                    .font(.acornButton)
                }
            }
        }
    }
}

struct ProviderRow: View {
    let metadata: ProviderMetadata
    let isActive: Bool
    let hasCredentials: Bool
    let requiresKey: Bool
    let unsupported: Bool
    var select: () -> Void

    var body: some View {
        Button(action: select) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(metadata.displayName)
                        .font(.acornBody.weight(.semibold))
                        .foregroundStyle(unsupported ? Color.secondary : Color.primary)
                    HStack(spacing: 6) {
                        Text(metadata.defaultModel)
                            .font(.acornCaption)
                            .foregroundStyle(.secondary)
                        if metadata.featured {
                            Text("Recommended")
                                .font(.acornCaption.weight(.semibold))
                                .foregroundStyle(Color.acornAccent)
                        }
                        if unsupported {
                            Text("iOS not supported")
                                .font(.acornCaption.weight(.semibold))
                                .foregroundStyle(.tertiary)
                        }
                        if requiresKey, !hasCredentials {
                            Text("Needs API key")
                                .font(.acornCaption.weight(.semibold))
                                .foregroundStyle(.tertiary)
                        }
                    }
                }
                Spacer()
                if isActive {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(Color.acornAccent)
                }
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(unsupported)
    }
}

struct CredentialsView: View {
    let metadata: ProviderMetadata
    @Environment(ProvidersStore.self) private var providers
    @Environment(\.dismiss) private var dismiss
    @State private var apiKey: String = ""
    @State private var revealed = false
    @State private var saving = false

    var body: some View {
        Form {
            Section("API key") {
                if revealed {
                    TextField("paste your \(metadata.displayName) key", text: $apiKey)
                    #if os(iOS)
                        .textInputAutocapitalization(.never)
                    #endif
                        .autocorrectionDisabled()
                } else {
                    SecureField("paste your \(metadata.displayName) key", text: $apiKey)
                }
                Toggle("Show key", isOn: $revealed)
            }
            Section {
                Button {
                    save()
                } label: {
                    HStack {
                        if saving { ProgressView() }
                        Text("Save key")
                    }
                }
                .disabled(apiKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || saving)

                if providers.hasCredentials[metadata.id] == true {
                    Button("Remove saved key", role: .destructive) {
                        _Concurrency.Task {
                            await providers.deleteCredentials(providerId: metadata.id)
                            dismiss()
                        }
                    }
                }
            }
            Section {
                Text("Keys are stored in the iOS Keychain (service `\(Keychain.acorn.service)`). They never leave your device unencrypted.")
                    .font(.acornCaption)
                    .foregroundStyle(.secondary)
            }
        }
        .navigationTitle(metadata.displayName)
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }

    private func save() {
        saving = true
        _Concurrency.Task {
            await providers.saveCredentials(
                providerId: metadata.id,
                apiKey: apiKey.trimmingCharacters(in: .whitespacesAndNewlines)
            )
            saving = false
            dismiss()
        }
    }
}
