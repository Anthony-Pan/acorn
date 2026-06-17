import WidgetKit
import SwiftUI

@main
struct AcornWidgetBundle: WidgetBundle {
    var body: some Widget {
        TodayWidget()
        if #available(iOS 16.2, *) {
            TaskActivityWidget()
        }
    }
}
