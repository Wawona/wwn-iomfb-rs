# GPU zero-copy present

The Mode B framebuffer path is **IOSurface == dma-buf**. Metal renders
into an IOSurface-backed texture. IOMFB `SwapSetLayer` receives that
same IOSurface. No CPU lock, no blit, no second allocation.

```text
TrollStore app / jailbreak tweak / Wawona compositor
  -> IOSurface (BGRA)
  -> Metal texture wrap (same backing, render target)
  -> iomfb::GpuSwapchain::present  or  present_external
  -> IOMFB layer 0
```

Linux `zwp_linux_dmabuf_v1` on Wawona already stores an IOSurface. When
Desktop Replacement switches to this crate, the compositor passes that
IOSurface into `present_external`. The IOSurfaceID must not change.

## Consumers

| Consumer | How it links | Privilege |
|---|---|---|
| TrollStore `.tipa` | static `iomfb-c` + `include/iomfb.h`, ldid IOMFB ents | No jailbreak. No ElleKit |
| Jailbreak tweak | same library, process already unsandboxed | Sileo / ElleKit is the tweak, not this crate |
| Wawona Mode B Desktop | later L4 call into `GpuSwapchain::present_external` | TrollStore or Sileo |

HID / SpringBoard park is **not** this crate. [`TouchMap`](../crates/iomfb/src/touch.rs)
only converts normalized points to the IOMFB pixel grid.

## Optimizations this crate owns

1. Triple-buffer (`SWAPCHAIN_BUFFERS == 3`). IOMFB layers are 0-3. We
   present on layer 0 and rotate our own IOSurfaces.
2. Metal wrap uses `newTextureWithDescriptor:iosurface:plane:`. Shared
   storage. Render-target + shader-read.
3. Present is set + end. `SwapWait` `0xe000002b` on guest 26.1 is
   incomplete wait, not a failed frame (`PresentStatus`).
4. External compositor surfaces skip the pool. No extra copy.
5. Default SpringBoard surface is restore-only. Never a render target.
6. Null / size / fourcc checks before `SwapSetLayer`.

A Metal blit is allowed only when a producer has **no** IOSurface. That
is a caller fallback. This crate will not insert one on the Desktop path.

## vphone vs device

vphone proves IOMFB swap and the TrollStore install path. It has no
Metal. `has_metal()` is false; acquire still returns IOSurfaces. Full
GPU proof is a physical TrollStore device.

## Evidence

```text
present: route=direct-iosurface copy=zero backing_id=<IOSurfaceID>
```

`iomfb_present_info.zero_copy` is 1 on this path.
