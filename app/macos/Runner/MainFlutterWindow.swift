import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    var windowFrame = self.frame
    self.contentViewController = flutterViewController
    // The top bar holds five area tabs, Undo and Redo, four quick actions,
    // Expert and Save; below 1100 points the actions cover the tabs. The
    // pages are laid out for 1280 x 800, so the window opens at least that
    // large (macOS keeps it on the screen) and never shrinks below 1100 x 700.
    self.contentMinSize = NSSize(width: 1100, height: 700)
    windowFrame.size = NSSize(width: max(windowFrame.width, 1280), height: max(windowFrame.height, 860))
    self.setFrame(windowFrame, display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)

    super.awakeFromNib()
  }
}
