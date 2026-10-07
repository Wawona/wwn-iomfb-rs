import Metal
import QuartzCore
import UIKit

@objc(BenchView)
final class BenchView: UIView {
  override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
    benchDelegate?.handleTouches(touches, state: Int32(IOMFB_TOUCH_DOWN), event: event)
  }

  override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
    benchDelegate?.handleTouches(touches, state: Int32(IOMFB_TOUCH_MOTION), event: event)
  }

  override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
    benchDelegate?.handleTouches(touches, state: Int32(IOMFB_TOUCH_UP), event: event)
  }

  override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
    benchDelegate?.handleTouches(touches, state: Int32(IOMFB_TOUCH_CANCEL), event: event)
  }

  private var benchDelegate: AppDelegate? {
    UIApplication.shared.delegate as? AppDelegate
  }
}

@objc(AppDelegate)
final class AppDelegate: NSObject, UIApplicationDelegate {
  var window: UIWindow?

  var link: CADisplayLink?
  var device: MTLDevice?
  var queue: MTLCommandQueue?
  var scenePipe: MTLRenderPipelineState?
  var hudPipe: MTLRenderPipelineState?
  var fieldPipe: MTLComputePipelineState?
  var uniformBuf: MTLBuffer?
  var particleBuf: MTLBuffer?
  var fieldTex: MTLTexture?
  var sceneTex: MTLTexture?
  var swapchain: UnsafeMutableRawPointer?
  var touch: UnsafeMutableRawPointer?
  var width: UInt32 = 0
  var height: UInt32 = 0
  var lastTime: Double = 0
  var fpsEma: Double = 60
  var lastPan = SIMD2<Float>(0, 0)
  var hasPan = false
  var pinch0: CGFloat = 0
  var benchView: BenchView?
  var u = tipa_bench_uniforms()

  func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
  ) -> Bool {
    _ = application
    _ = launchOptions

    freopen("/tmp/iomfb-bench.log", "w", stderr)
    setvbuf(stderr, nil, _IONBF, 0)

    let win = UIWindow(frame: UIScreen.main.bounds)
    let view = BenchView(frame: win.bounds)
    view.isMultipleTouchEnabled = true
    view.backgroundColor = .black
    benchView = view
    let vc = UIViewController()
    vc.view = view
    win.rootViewController = vc
    win.makeKeyAndVisible()
    window = win

    let nb = UIScreen.main.nativeBounds
    iomfb_channel_set(Int32(IOMFB_CHANNEL_TROLLSTORE))
    iomfb_display_configure(UInt32(nb.size.width), UInt32(nb.size.height))

    var sw: UnsafeMutableRawPointer?
    let rc = iomfb_swapchain_open(&sw)
    let swPtr = sw
    let userland = swPtr.map { iomfb_swapchain_is_userland($0) != 0 ? 1 : 0 } ?? 0
    let hasMetal = swPtr.map { iomfb_swapchain_has_metal($0) != 0 ? 1 : 0 } ?? 0
    fputs(
      "iomfb_swapchain_open rc=\(rc) userland=\(userland) bound=\(iomfb_bound_export_count()) has_metal=\(hasMetal)\n",
      stderr
    )
    guard rc == IOMFB_C_OK, let chain = sw else { return true }

    iomfb_swapchain_set_present(chain, tipa_bench_present_fn(), Unmanaged.passUnretained(view).toOpaque())
    swapchain = chain
    var w: UInt32 = 0
    var h: UInt32 = 0
    iomfb_swapchain_size(chain, &w, &h)
    width = w
    height = h

    var touchHandle: UnsafeMutableRawPointer?
    iomfb_touch_open_swapchain(chain, &touchHandle)
    touch = touchHandle
    if let touchHandle {
      iomfb_touch_set_view(touchHandle, win.bounds.size.width, win.bounds.size.height)
    }

    guard setupMetal(width: w, height: h) else { return true }

    memset(&u, 0, MemoryLayout<tipa_bench_uniforms>.size)
    u.res = SIMD2<Float>(Float(w), Float(h))
    u.cam = SIMD2<Float>(0.35, 0.18)
    u.load = 1.0
    u.steps = 96
    u.touch_n = 1
    setTouch(0, SIMD4<Float>(0.5, 0.42, 1, 1))
    lastTime = tipa_bench_now_sec()
    fpsEma = 60

    let displayLink = CADisplayLink(target: self, selector: #selector(tick))
    displayLink.preferredFramesPerSecond = 60
    displayLink.add(to: .main, forMode: .common)
    link = displayLink
    fputs("bench start iosurface_zero_copy=1\n", stderr)
    DispatchQueue.main.async { [weak self] in
      fputs("first tick dispatch\n", stderr)
      self?.tick()
    }
    return true
  }

  func handleTouches(_ touches: Set<UITouch>, state: Int32, event: UIEvent?) {
    _ = event
    guard let touchHandle = touch, let win = window else { return }
    let vs = win.bounds.size
    iomfb_touch_set_view(touchHandle, vs.width, vs.height)

    for t in touches {
      let p = t.location(in: win)
      var ev = iomfb_touch_event()
      memset(&ev, 0, MemoryLayout<iomfb_touch_event>.size)
      let injectRc = iomfb_touch_inject(
        touchHandle,
        Int32(truncatingIfNeeded: t.hash),
        state,
        p.x,
        p.y,
        Int32(IOMFB_TOUCH_SPACE_VIEW),
        &ev
      )
      if injectRc != IOMFB_C_OK { continue }

      let slot = Int(ev.slot)
      if slot < Int(TIPA_BENCH_MAX_TOUCH) {
        if state == IOMFB_TOUCH_UP || state == IOMFB_TOUCH_CANCEL {
          setTouch(slot, SIMD4<Float>(Float(ev.nx), Float(ev.ny), 0, 0))
        } else {
          setTouch(slot, SIMD4<Float>(Float(ev.nx), Float(ev.ny), 1, 1))
        }
      }

      if state == IOMFB_TOUCH_DOWN {
        lastPan = SIMD2<Float>(Float(p.x), Float(p.y))
        hasPan = true
      } else if state == IOMFB_TOUCH_MOTION, hasPan, touches.count == 1 {
        let dx = Float(p.x) - lastPan.x
        let dy = Float(p.y) - lastPan.y
        u.cam.x += dx * 0.008
        u.cam.y = min(1.1, max(-0.35, u.cam.y - dy * 0.008))
        lastPan = SIMD2<Float>(Float(p.x), Float(p.y))
      }
    }

    if touches.count >= 2 {
      let all = Array(touches)
      if all.count >= 2 {
        let a = all[0].location(in: win)
        let b = all[1].location(in: win)
        let dist = hypot(a.x - b.x, a.y - b.y)
        if pinch0 < 1.0 {
          pinch0 = dist
        } else {
          let scale = Float(dist / pinch0)
          u.load = min(2.2, max(0.45, u.load * (0.7 + 0.3 * scale)))
          pinch0 = dist
        }
      }
    } else {
      pinch0 = 0
    }

    if state == IOMFB_TOUCH_UP || state == IOMFB_TOUCH_CANCEL, touches.count <= 1 {
      hasPan = false
    }

    var n: UInt32 = 0
    for i in 0..<Int(TIPA_BENCH_MAX_TOUCH) {
      if touchWeight(i) > 0.5 { n += 1 }
    }
    u.touch_n = n
  }

  private func setTouch(_ index: Int, _ value: SIMD4<Float>) {
    withUnsafeMutablePointer(to: &u.touches) { ptr in
      ptr.withMemoryRebound(to: SIMD4<Float>.self, capacity: Int(TIPA_BENCH_MAX_TOUCH)) { slots in
        slots[index] = value
      }
    }
  }

  private func touchWeight(_ index: Int) -> Float {
    withUnsafePointer(to: u.touches) { ptr in
      ptr.withMemoryRebound(to: SIMD4<Float>.self, capacity: Int(TIPA_BENCH_MAX_TOUCH)) { slots in
        slots[index].w
      }
    }
  }
}

@_cdecl("tipa_bench_present_swift")
func tipaBenchPresentSwift(
  ctx: UnsafeMutableRawPointer?,
  surface: UnsafeMutableRawPointer?,
  w: UInt32,
  h: UInt32,
  token: Int32,
  layer: Int32
) {
  _ = w
  _ = h
  _ = token
  _ = layer
  guard let ctx, let surface else { return }
  let view = Unmanaged<BenchView>.fromOpaque(ctx).takeUnretainedValue()
  view.layer.contents = Unmanaged<AnyObject>.fromOpaque(surface).takeUnretainedValue()
}
