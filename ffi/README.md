# ffi (C + Swift Metal glue)

Thin glue:

- `iomfb_trampoline.c`: rect packing into the C ABI
- `iomfb_surface.c`: `IOSurfaceCreate` (CoreFoundation)
- `iomfb_metal.swift`: Metal
  `newTextureWithDescriptor:iosurface:plane:` (zero-copy wrap),
  command queue, `waitUntilCompleted`, GPU clear into the same surface

Policy, swapchain, and IOMFB present live in Rust. Do not blit here.
Do not include Apple IOMFB headers. `dlopen` the framework at runtime.
