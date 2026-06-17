import SwiftUI

public struct AcornCardModifier: ViewModifier {
    public init() {}

    public func body(content: Content) -> some View {
        content
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .fill(.background.shadow(.inner(color: Color.acornAccent.opacity(0.05), radius: 2)))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .stroke(Color.acornCardBorder, lineWidth: 1)
            )
            .shadow(color: Color.acornBark.opacity(0.08), radius: 6, x: 0, y: 3)
    }
}

public extension View {
    func acornCard() -> some View {
        modifier(AcornCardModifier())
    }
}
