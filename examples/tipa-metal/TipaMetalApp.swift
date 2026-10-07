import SwiftUI
import UIKit

@main
struct TipaMetalApp: App {
  @UIApplicationDelegateAdaptor(TipaMetalDelegate.self) private var delegate

  var body: some Scene {
    WindowGroup {
      Color(red: 0.95, green: 0.15, blue: 0.85)
        .ignoresSafeArea()
    }
  }
}

final class TipaMetalDelegate: NSObject, UIApplicationDelegate {
  func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
  ) -> Bool {
    _ = application
    _ = launchOptions
    freopen("/tmp/iomfb-metal.log", "w", stderr)
    setvbuf(stderr, nil, _IONBF, 0)
    let rc = tipa_metal_present_frame()
    fputs("present_metal_frame rc=\(rc)\n", stderr)
    return true
  }
}

@_silgen_name("tipa_metal_present_frame")
func tipa_metal_present_frame() -> Int32
