// Prints the CGWindowID of the frontmost on-screen window owned by the
// given process name (default "netpeek"), for feeding to
// `screencapture -l<windowid>`. The Accessibility API (System Events)
// doesn't expose CGWindowID, so this goes straight to CoreGraphics.
import CoreGraphics
import Foundation

let options = CGWindowListOption(arrayLiteral: .optionOnScreenOnly)
guard let windowList = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as NSArray? as? [[String: AnyObject]] else {
    exit(1)
}

let target = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "netpeek"

for window in windowList {
    let owner = window[kCGWindowOwnerName as String] as? String ?? ""
    if owner.lowercased() == target.lowercased() {
        let windowID = window[kCGWindowNumber as String] as? Int ?? -1
        let name = window[kCGWindowName as String] as? String ?? ""
        let bounds = window[kCGWindowBounds as String] as? [String: Any] ?? [:]
        print("id=\(windowID) name=\(name) bounds=\(bounds)")
    }
}
