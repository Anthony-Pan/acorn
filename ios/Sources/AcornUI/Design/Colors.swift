import SwiftUI

public extension Color {
    static let acornBark = Color(red: 0.545, green: 0.271, blue: 0.075)
    static let autumnLeaf = Color(red: 0.710, green: 0.502, blue: 0.247)
    static let warmCream = Color(red: 0.984, green: 0.969, blue: 0.929)
    static let autumnAmber = Color(red: 0.910, green: 0.667, blue: 0.310)
    static let mossGreen = Color(red: 0.404, green: 0.502, blue: 0.282)
    static let dustyRose = Color(red: 0.788, green: 0.471, blue: 0.443)
    static let nutShadow = Color(red: 0.302, green: 0.149, blue: 0.063)

    static let acornCardBackground = Color(light: warmCream, dark: nutShadow.opacity(0.7))
    static let acornCardBorder = Color(light: autumnLeaf.opacity(0.25), dark: autumnLeaf.opacity(0.4))
    static let acornAccent = autumnLeaf

    static func priorityColor(_ p: Priority) -> Color {
        switch p {
        case .high: return .dustyRose
        case .medium: return .autumnAmber
        case .low: return .mossGreen
        }
    }
}

import AcornCore

public extension Color {
    init(light: Color, dark: Color) {
        #if canImport(UIKit)
        self.init(uiColor: UIColor(dynamicProvider: { trait in
            trait.userInterfaceStyle == .dark ? UIColor(dark) : UIColor(light)
        }))
        #else
        self = light
        #endif
    }
}
