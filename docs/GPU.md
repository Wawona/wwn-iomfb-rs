# GPU zero-copy present

The Mode B framebuffer path is **IOSurface == dma-buf**. Metal renders
into an IOSurface-backed texture. The product backend presents that
same IOSurface through a host callback (CALayer / iland). It does not
`IOConnect` the IOMFB userclient. See `docs/USERLAND.md`.

```text
TrollStore app / jailbreak tweak / Wawona compositor
  -> IOSurface (BGRA)
  -> Metal texture wrap (same backing, render target)
  -> iomfb::GpuSwapchain::present  or  present_external
  -> present callback (CALayer / iland)
```

Linux `zwp_linux_dmabuf_v1` on Wawona already stores an IOSurface. When
Desktop Replacement switches to this crate, the compositor passes that
IOSurface into `present_external`. The IOSurfaceID must not change.

## Consumers

| Consumer | How it links | Privilege |
|---|---|---|
| TrollStore `.tipa` | static `iomfb-c` (no `apple-iomfb`), IOSurface/Metal ents | Limited. No IOMFB userclient |
| Jailbreak CLI / tweak | `iomfb-c` + `apple-iomfb`, `IOMFB_CHANNEL_JAILBREAK` | Full RE. ElleKit is the tweak, not this crate |
| Wawona Mode B Desktop | later L4 call into `GpuSwapchain::present_external` | TrollStore or Sileo |

HID / SpringBoard park is **not** this crate. [`TouchSeat`](../crates/iomfb/src/touch.rs)
maps host points onto the IOMFB pixel grid (normalized, view, pixel, or
HID) and tracks 16 slots with the same state numbers as Wawona Mode B
(`0` up, `1` down, `2` motion, `3` cancel). Callers still supply the
points. `iomfb_touch_inject` is the C entry.

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

## vphone is the proof device

vphone `wawona-jb` (iPhone99,11, iOS 26.1) ships `Metal.framework` and
IOMFB. The Metal tipa (`scripts/build-tipa-metal.sh`) encodes a GPU
clear into an IOSurface and swaps that same ID. `has_metal()` is true
when `MTLCreateSystemDefaultDevice` returns a device. Do not defer
this path to a physical iPhone.

## Evidence

vphone `wawona-jb` Metal tipa (`scripts/build-tipa-metal.sh`), SSH launch plus
sock screenshot `.agent-device/test-artifacts/iomfb-metal-vphone.png`
(full-frame magenta present):

```text
bind 153/153
iogpu_match IOGPU kr=0 count=1
iogpu_match AppleParavirtGPU kr=0 count=1
iogpu_match IOMobileFramebuffer kr=0 count=1
display 1290x2796
RequestPowerChange(1)=0
create/import: backing_id=28
MTLCreateSystemDefaultDevice=Apple Paravirtual device GPU
has_metal=1 name=Apple Paravirtual device GPU
metal wrap+clear ok iosurface_id=28
present: route=direct-iosurface copy=zero backing_id=28 has_metal=1 metal_ok=1 set=0 end=0 wait=0 token=1242
```

UIKit launch is required. SSH `main()` without UIKit can see
`has_metal=0` on the same guest. `IOGPUDevice` count stays 0;
`IOGPU` + `AppleParavirtGPU` are the live services. Sock shot
`.agent-device/test-artifacts/iomfb-metal-vphone.png` is full-frame
magenta on the same IOSurfaceID. Do not add a physical-only branch.

`iomfb_present_info.zero_copy` is 1 on this path.

Heavy Metal bench tipa: `scripts/build-tipa-bench.sh` (bundle
`com.aspauldingcode.wawona.iomfb.bench`). Product `iomfb-c` (no
`apple-iomfb` feature). Logs `/tmp/iomfb-bench.log`. Guest 2026-09-04:
`userland=1 bound=0 has_metal=1`, ~60 FPS at 1290x2796, `zero=1`,
same IOSurface pool ID on the 15-frame cadence. Present is
`CALayer.contents`. Pinch raises march load.
