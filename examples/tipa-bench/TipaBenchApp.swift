import UIKit

@main
enum TipaBenchMain {
  static func main() {
    autoreleasepool {
      let argc = CommandLine.argc
      let argv = CommandLine.unsafeArgv
      exit(UIApplicationMain(argc, argv, nil, "AppDelegate"))
    }
  }
}
