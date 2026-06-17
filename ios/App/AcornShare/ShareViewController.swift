import UIKit
import Social
import UniformTypeIdentifiers
import AcornCore

final class ShareViewController: SLComposeServiceViewController {
    override func presentationAnimationDidFinish() {
        super.presentationAnimationDidFinish()
        title = "Stash to Acorn 🌰"
        placeholder = "Anything you want Acorn to break into tasks…"
        let current = contentText ?? ""
        if let initial = extractedText, current.isEmpty {
            textView.text = initial
            validateContent()
        }
    }

    override func isContentValid() -> Bool {
        !(contentText ?? "").trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    override func didSelectPost() {
        let text = contentText ?? ""
        PendingStashQueue.enqueue(text: text)
        extensionContext?.completeRequest(returningItems: [], completionHandler: nil)
    }

    override func configurationItems() -> [Any]! {
        []
    }

    private var extractedText: String? {
        guard let item = extensionContext?.inputItems.first as? NSExtensionItem else { return nil }
        return item.attributedContentText?.string ?? item.attributedTitle?.string
    }
}
