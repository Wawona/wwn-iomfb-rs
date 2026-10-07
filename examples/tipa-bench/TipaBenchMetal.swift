import Metal
import UIKit

extension AppDelegate {
  func setupMetal(width w: UInt32, height h: UInt32) -> Bool {
    guard let dev = MTLCreateSystemDefaultDevice() else {
      fputs("MTLCreateSystemDefaultDevice=nil\n", stderr)
      return false
    }
    device = dev
    fputs("MTLCreateSystemDefaultDevice=\(dev.name) display=\(w)x\(h)\n", stderr)

    guard let q = dev.makeCommandQueue() else { return false }
    queue = q

    guard let libPath = Bundle.main.path(forResource: "default", ofType: "metallib") else {
      fputs("metallib path missing\n", stderr)
      return false
    }
    do {
      let lib = try dev.makeLibrary(filepath: libPath)
      let sceneDesc = MTLRenderPipelineDescriptor()
      sceneDesc.vertexFunction = lib.makeFunction(name: "vs_fullscreen")
      sceneDesc.fragmentFunction = lib.makeFunction(name: "fs_scene")
      sceneDesc.colorAttachments[0].pixelFormat = .bgra8Unorm
      scenePipe = try dev.makeRenderPipelineState(descriptor: sceneDesc)

      let hudDesc = MTLRenderPipelineDescriptor()
      hudDesc.vertexFunction = lib.makeFunction(name: "vs_fullscreen")
      hudDesc.fragmentFunction = lib.makeFunction(name: "fs_hud")
      hudDesc.colorAttachments[0].pixelFormat = .bgra8Unorm
      hudPipe = try dev.makeRenderPipelineState(descriptor: hudDesc)

      guard let cs = lib.makeFunction(name: "cs_field") else { return false }
      fieldPipe = try dev.makeComputePipelineState(function: cs)
    } catch {
      fputs("pipeline failed: \((error as NSError).localizedDescription)\n", stderr)
      return false
    }

    guard scenePipe != nil, hudPipe != nil, fieldPipe != nil else { return false }

    let fd = MTLTextureDescriptor.texture2DDescriptor(
      pixelFormat: .rgba16Float,
      width: Int(TIPA_BENCH_FIELD_W),
      height: Int(TIPA_BENCH_FIELD_H),
      mipmapped: false
    )
    fd.usage = [.shaderRead, .shaderWrite]
    fd.storageMode = .shared
    fieldTex = dev.makeTexture(descriptor: fd)

    let sd = MTLTextureDescriptor.texture2DDescriptor(
      pixelFormat: .bgra8Unorm,
      width: Int(w),
      height: Int(h),
      mipmapped: false
    )
    sd.usage = [.shaderRead, .renderTarget]
    sd.storageMode = .shared
    sceneTex = dev.makeTexture(descriptor: sd)

    uniformBuf = dev.makeBuffer(length: MemoryLayout<tipa_bench_uniforms>.size, options: .storageModeShared)
    let plen = MemoryLayout<tipa_bench_particle>.size * Int(TIPA_BENCH_PART_COUNT)
    particleBuf = dev.makeBuffer(length: plen, options: .storageModeShared)

    if let particleBuf {
      particleBuf.contents().withMemoryRebound(to: tipa_bench_particle.self, capacity: Int(TIPA_BENCH_PART_COUNT)) { ps in
        for i in 0..<Int(TIPA_BENCH_PART_COUNT) {
          let x = Float(i % Int(TIPA_BENCH_FIELD_W)) / Float(TIPA_BENCH_FIELD_W)
          let y = Float(i / Int(TIPA_BENCH_FIELD_W)) / Float(TIPA_BENCH_FIELD_H)
          ps[i].p = SIMD2<Float>(x, y)
          ps[i].v = SIMD2<Float>(0, 0)
        }
      }
    }
    return true
  }

  @objc func tick() {
    guard let swapchain, let queue, let uniformBuf else { return }

    let t0 = tipa_bench_now_sec()
    let dt = Float(max(1.0 / 240.0, t0 - lastTime))
    lastTime = t0
    fpsEma = fpsEma * 0.9 + (1.0 / Double(dt)) * 0.1
    u.time += dt
    u.dt = dt
    u.fps = Float(fpsEma)
    u.cpu_load = tipa_bench_cpu_load_sample()
    let th = ProcessInfo.processInfo.thermalState
    u.thermal = UInt32(th.rawValue)
    u.steps = 64 + UInt32(u.load * 64)
    if u.steps > 192 { u.steps = 192 }
    u.frame &+= 1

    uniformBuf.contents().copyMemory(from: &u, byteCount: MemoryLayout<tipa_bench_uniforms>.size)

    var surf: UnsafeMutableRawPointer?
    var metal: UnsafeMutableRawPointer?
    var sid: UInt32 = 0
    let arc = iomfb_swapchain_acquire(swapchain, &surf, &metal, &sid)
    guard arc == IOMFB_C_OK, let metal else {
      if u.frame % 120 == 0 {
        fputs("acquire rc=\(arc) metal=\(String(describing: metal))\n", stderr)
      }
      return
    }
    let presentTex = Unmanaged<MTLTexture>.fromOpaque(metal).takeUnretainedValue()

    guard let buf = queue.makeCommandBuffer(),
      let fieldPipe,
      let fieldTex,
      let particleBuf,
      let sceneTex,
      let scenePipe,
      let hudPipe
    else { return }

    if let enc = buf.makeComputeCommandEncoder() {
      enc.setComputePipelineState(fieldPipe)
      enc.setTexture(fieldTex, index: 0)
      enc.setBuffer(particleBuf, offset: 0, index: 0)
      enc.setBuffer(uniformBuf, offset: 0, index: 1)
      let grid = MTLSize(width: Int(TIPA_BENCH_FIELD_W), height: Int(TIPA_BENCH_FIELD_H), depth: 1)
      let tw = fieldPipe.threadExecutionWidth
      var thCount = fieldPipe.maxTotalThreadsPerThreadgroup / tw
      if thCount < 1 { thCount = 1 }
      enc.dispatchThreads(grid, threadsPerThreadgroup: MTLSize(width: tw, height: thCount, depth: 1))
      enc.endEncoding()
    }

    do {
      let pass = MTLRenderPassDescriptor()
      pass.colorAttachments[0].texture = sceneTex
      pass.colorAttachments[0].loadAction = .clear
      pass.colorAttachments[0].storeAction = .store
      pass.colorAttachments[0].clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 1)
      if let enc = buf.makeRenderCommandEncoder(descriptor: pass) {
        enc.setRenderPipelineState(scenePipe)
        enc.setFragmentBuffer(uniformBuf, offset: 0, index: 0)
        enc.setFragmentTexture(fieldTex, index: 0)
        enc.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: 3)
        enc.endEncoding()
      }
    }

    do {
      let pass = MTLRenderPassDescriptor()
      pass.colorAttachments[0].texture = presentTex
      pass.colorAttachments[0].loadAction = .dontCare
      pass.colorAttachments[0].storeAction = .store
      if let enc = buf.makeRenderCommandEncoder(descriptor: pass) {
        enc.setRenderPipelineState(hudPipe)
        enc.setFragmentBuffer(uniformBuf, offset: 0, index: 0)
        enc.setFragmentTexture(sceneTex, index: 0)
        enc.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: 3)
        enc.endEncoding()
      }
    }

    buf.commit()
    buf.waitUntilCompleted()

    var gpu: Double = 0
    if buf.gpuEndTime > 0, buf.gpuStartTime > 0 {
      gpu = (buf.gpuEndTime - buf.gpuStartTime) * 1000
    }
    u.gpu_ms = Float(gpu)

    var info = iomfb_present_info()
    memset(&info, 0, MemoryLayout<iomfb_present_info>.size)
    let prc = iomfb_swapchain_present(swapchain, &info)
    let t1 = tipa_bench_now_sec()
    u.cpu_ms = Float((t1 - t0) * 1000)

    if u.frame % 15 == 0 {
      let thName = thermalName(th)
      fputs(
        """
        frame=\(u.frame) fps=\(String(format: "%.1f", u.fps)) gpu_ms=\(String(format: "%.2f", u.gpu_ms)) \
        cpu_ms=\(String(format: "%.2f", u.cpu_ms)) thermal=\(thName)(\(u.thermal)) \
        cpu=\(String(format: "%.0f", u.cpu_load * 100))% load=\(String(format: "%.2f", u.load)) \
        steps=\(u.steps) touches=\(u.touch_n) sid=\(sid) zero=\(info.zero_copy) \
        setend=\(prc) wait=\(info.wait_rc) token=\(info.token)
        """,
        stderr
      )
    }
  }
}

private func thermalName(_ s: ProcessInfo.ThermalState) -> String {
  switch s {
  case .nominal: return "nominal"
  case .fair: return "fair"
  case .serious: return "serious"
  case .critical: return "critical"
  @unknown default: return "unknown"
  }
}
