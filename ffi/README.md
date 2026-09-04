# ffi (ObjC trampoline)

Thin trampoline:

- `iomfb_trampoline.m`: `CGRect` packing into the C ABI
- `iomfb_surface.m`: `IOSurfaceCreate` and Metal
  `newTextureWithDescriptor:iosurface:plane:` (zero-copy wrap)

Policy, swapchain, and IOMFB present live in Rust. Do not blit here.
Do not include Apple IOMFB headers. `dlopen` the framework at runtime.
