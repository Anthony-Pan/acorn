import SwiftUI

public extension Font {
    static let acornTitle = Font.system(.largeTitle, design: .serif, weight: .semibold)
    static let acornHeader = Font.system(.title2, design: .rounded, weight: .semibold)
    static let acornBody = Font.system(.body, design: .rounded)
    static let acornCaption = Font.system(.caption, design: .rounded).monospacedDigit()
    static let acornButton = Font.system(.headline, design: .rounded, weight: .semibold)
}
