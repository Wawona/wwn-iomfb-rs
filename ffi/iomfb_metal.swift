import CoreFoundation
import IOSurface
import Metal

private func fourCharCode(_ s: String) -> UInt32 {
    var r: UInt32 = 0
    for u in s.utf8.prefix(4) {
        r = (r << 8) | UInt32(u)
    }
    return r
}
#if canImport(UIKit)
import UIKit
#endif

@_cdecl("iomfb_glue_metal_device")
public func iomfbGlueMetalDevice() -> UnsafeMutableRawPointer? {
    guard let device = MTLCreateSystemDefaultDevice() else { return nil }
    return Unmanaged.passRetained(device).toOpaque()
}

@_cdecl("iomfb_glue_metal_release")
public func iomfbGlueMetalRelease(_ object: UnsafeMutableRawPointer?) {
    guard let object else { return }
    Unmanaged<AnyObject>.fromOpaque(object).release()
}

@_cdecl("iomfb_glue_metal_texture_wrap")
public func iomfbGlueMetalTextureWrap(
    _ devicePtr: UnsafeMutableRawPointer?,
    _ surfacePtr: UnsafeMutableRawPointer?,
    _ renderTarget: Int32
) -> UnsafeMutableRawPointer? {
    guard let devicePtr, let surfacePtr else { return nil }
    let device = Unmanaged<MTLDevice>.fromOpaque(devicePtr).takeUnretainedValue()
    let surface = Unmanaged<IOSurface>.fromOpaque(surfacePtr).takeUnretainedValue()
    let fourcc = IOSurfaceGetPixelFormat(surface)
    let fmt: MTLPixelFormat
    if fourcc == fourCharCode("BGRA") {
        fmt = .bgra8Unorm
    } else {
        return nil
    }
    let w = IOSurfaceGetWidth(surface)
    let h = IOSurfaceGetHeight(surface)
    let td = MTLTextureDescriptor.texture2DDescriptor(
        pixelFormat: fmt,
        width: w,
        height: h,
        mipmapped: false
    )
    td.storageMode = .shared
    td.usage = [.shaderRead]
    if renderTarget != 0 {
        td.usage.insert(.renderTarget)
    }
    guard let tex = device.makeTexture(descriptor: td, iosurface: surface, plane: 0) else {
        return nil
    }
    return Unmanaged.passRetained(tex).toOpaque()
}

@_cdecl("iomfb_glue_metal_queue")
public func iomfbGlueMetalQueue(_ devicePtr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let devicePtr else { return nil }
    let device = Unmanaged<MTLDevice>.fromOpaque(devicePtr).takeUnretainedValue()
    guard let queue = device.makeCommandQueue() else { return nil }
    return Unmanaged.passRetained(queue).toOpaque()
}

@_cdecl("iomfb_glue_metal_wait")
public func iomfbGlueMetalWait(_ queuePtr: UnsafeMutableRawPointer?) -> Int32 {
    guard let queuePtr else { return -1 }
    let queue = Unmanaged<MTLCommandQueue>.fromOpaque(queuePtr).takeUnretainedValue()
    guard let buf = queue.makeCommandBuffer() else { return -1 }
    buf.commit()
    buf.waitUntilCompleted()
    return 0
}

@_cdecl("iomfb_glue_metal_clear")
public func iomfbGlueMetalClear(
    _ queuePtr: UnsafeMutableRawPointer?,
    _ texturePtr: UnsafeMutableRawPointer?,
    _ r: Float,
    _ g: Float,
    _ b: Float,
    _ a: Float
) -> Int32 {
    guard let queuePtr, let texturePtr else { return -1 }
    let queue = Unmanaged<MTLCommandQueue>.fromOpaque(queuePtr).takeUnretainedValue()
    let tex = Unmanaged<MTLTexture>.fromOpaque(texturePtr).takeUnretainedValue()
    guard let buf = queue.makeCommandBuffer() else { return -1 }
    let pass = MTLRenderPassDescriptor()
    pass.colorAttachments[0].texture = tex
    pass.colorAttachments[0].loadAction = .clear
    pass.colorAttachments[0].storeAction = .store
    pass.colorAttachments[0].clearColor = MTLClearColorMake(Double(r), Double(g), Double(b), Double(a))
    guard let enc = buf.makeRenderCommandEncoder(descriptor: pass) else { return -1 }
    enc.endEncoding()
    buf.commit()
    buf.waitUntilCompleted()
    return 0
}

@_cdecl("iomfb_glue_host_size")
public func iomfbGlueHostSize(
    _ width: UnsafeMutablePointer<UInt32>?,
    _ height: UnsafeMutablePointer<UInt32>?
) -> Int32 {
#if canImport(UIKit)
    let b = UIScreen.main.nativeBounds
    let w = UInt32(b.size.width)
    let h = UInt32(b.size.height)
    if w == 0 || h == 0 { return -1 }
    width?.pointee = w
    height?.pointee = h
    return 0
#else
    _ = width
    _ = height
    return -1
#endif
}
