# macOS Services menu integration plan

The Services menu (Apple Menu → Services, or right-click → Services on any
selected text) is the second half of "feels like Siri" — any text the user
selects in any app can be sent to Acorn with one click.

This document is a **v1.1 design** alongside the App Intents work. It is not
wired into the build yet because the same codesign requirement applies.

## What the user sees

1. Select some text in any macOS app (Safari, Notes, Slack, terminal, …).
2. Right-click → Services → **Stash to Acorn**.
3. Acorn opens (or comes forward) and breaks the text into task cards.

## Implementation plan

### 1. Info.plist fragment

Add this block to the bundled `Info.plist`. Tauri exposes a path for extra
plist entries via `tauri.conf.json` → `bundle.macOS.infoPlist`:

```xml
<key>NSServices</key>
<array>
    <dict>
        <key>NSMenuItem</key>
        <dict>
            <key>default</key>
            <string>Stash to Acorn</string>
        </dict>
        <key>NSMessage</key>
        <string>stashSelectedText</string>
        <key>NSPortName</key>
        <string>Acorn</string>
        <key>NSSendTypes</key>
        <array>
            <string>NSStringPboardType</string>
            <string>public.utf8-plain-text</string>
        </array>
        <key>NSRequiredContext</key>
        <dict>
            <key>NSTextContent</key>
            <string>PlainText</string>
        </dict>
    </dict>
</array>
```

### 2. Service handler

The Swift package in this directory is the right place to add the
`NSService` implementation. Add this file at
`Sources/AcornIntents/AcornServices.swift`:

```swift
import AppKit
import Foundation

public final class AcornServices: NSObject {
    @objc public func stashSelectedText(
        _ pboard: NSPasteboard,
        userData: String?,
        error: AutoreleasingUnsafeMutablePointer<NSString>
    ) {
        guard let text = pboard.string(forType: .string),
              !text.isEmpty,
              let encoded = text.addingPercentEncoding(
                  withAllowedCharacters: .urlQueryAllowed
              ),
              let url = URL(string: "acorn://stash?text=\(encoded)")
        else {
            error.pointee = "Acorn could not read the selection." as NSString
            return
        }
        NSWorkspace.shared.open(url)
    }

    @MainActor public static func register() {
        let provider = AcornServices()
        NSApp.servicesProvider = provider
        NSUpdateDynamicServices()
    }
}
```

### 3. Register on launch

In Tauri's Rust setup, after the app finishes launching, call into the Swift
package via FFI to invoke `AcornServices.register()`. The exact glue lives
in the v1.1 build doc.

### 4. Codesign and notarise

Same flow as App Intents. Unsigned Service providers are silently ignored
by the system Services menu.

## Why route through `acorn://` again

`AcornServices.stashSelectedText` could in principle drive the Rust core
directly through an embedded XPC socket. It deliberately does not — every
external entry point (Shortcuts, Services, Raycast, the terminal, deep
links from web pages) routes through the same `acorn://` URL scheme and the
single Rust `wire_deep_link` handler. Easier to test, easier to debug, no
extra IPC surface to maintain.

## See also

- `README.md` (this directory) — App Intents counterpart.
- `../docs/v1.1-shortcuts-build.md` — v1.1 build pipeline.
- `../src-tauri/src/lib.rs` `wire_deep_link` — the Rust receiver.
