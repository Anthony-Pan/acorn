import Foundation
#if canImport(AudioToolbox)
import AudioToolbox
#endif

public enum AcornSound: UInt32 {
    case chime = 1054
    case ping = 1057
    case warning = 1073
    case error = 1078
}

public enum Sounds {
    nonisolated(unsafe) public static var muted: Bool = false

    public static func play(_ sound: AcornSound) {
        guard !muted else { return }
        #if canImport(AudioToolbox)
        AudioServicesPlaySystemSound(SystemSoundID(sound.rawValue))
        #endif
    }
}
