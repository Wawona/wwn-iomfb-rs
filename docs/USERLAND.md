# Userspace IOMFB (TrollStore channel)

The TrollStore channel implements the confirmed IOMFB *ABI* in process
memory. It does not open `IOMobileFramebufferUserClient` and does not
call `IOConnect`. Jailbreak full RE is `docs/CHANNELS.md`.

```text
Metal / compositor
  -> IOSurface (dma-buf)
  -> Display::present_iosurface
  -> optional present callback (CALayer, iland, CAMetalLayer)
```

Apple `dlopen` is the jailbreak channel only. Set
`iomfb_channel_set(IOMFB_CHANNEL_JAILBREAK)` (or `WWN_IOMFB_CHANNEL=jailbreak`)
for `iomfb-live` / `iomfb-matrix`. TrollStore callers use
`Display::trollstore(w, h)` or `iomfb_display_open_trollstore`.

`Display::main()` resolves size from, in order:

1. `iomfb_display_configure` / `configure_userland`
2. `WWN_IOMFB_WIDTH` / `WWN_IOMFB_HEIGHT`
3. Public `UIScreen.nativeBounds` (iOS family glue)

HDCP, factory, calibration, and `KernelTests` return `Error::Absent`
on the userspace backend. Those families exist to talk to hardware.

`InstallVirtualDisplay(s)` stays fail-closed. This crate does not
invent a virt vtable.

Wawona L4 Desktop links `wwn_iomfb_*` (`include/wwn_iomfb.h`). That
open is Apple `GetMainDisplay` (own-display), not this userspace
path. Generic tipas stay on `iomfb_display_open_trollstore` /
`GpuSwapchain::main`.

## Guest proof (vphone `wawona-jb`, 2026-09-04)

`iomfb-userland` is built **without** `apple-iomfb` and signed **without**
`IOMobileFramebufferUserClient`. The Mach-O has no
`IOMobileFramebuffer.framework` load path.

```text
acquire surface=0x101426bf0 metal=0x10142db20 sid=4 has_metal=1
userland=1 bound=0 has_metal=1 sid=4 zero=1 token=1 wait=0 presents=1
PASS userspace no-IOConnect
```

Bench tipa build 31 (`com.aspauldingcode.wawona.iomfb.bench`), TrollStore
container install, `uiopen wawona-iomfb-bench://`. No IOMFB userclient
entitlement. `/tmp/iomfb-bench.log`:

```text
iomfb_swapchain_open rc=0 userland=1 bound=0 has_metal=1
MTLCreateSystemDefaultDevice=Apple Paravirtual device GPU display=1290x2796
frame=15 fps=49.4 gpu_ms=10.29 cpu_ms=14.52 … sid=26 zero=1 setend=0 wait=0 token=15
frame=750 fps=60.0 gpu_ms=7.20 cpu_ms=8.08 … sid=26 zero=1 setend=0 wait=0 token=750
```

`sid` repeats every 15 frames because the triple-buffer period divides
15. `bound=0` means the process never `dlopen`ed Apple IOMFB.
SpringBoard still composites the app window.

Lab oracles (`iomfb-live`, `iomfb-matrix`, `tipa-smoke`, `tipa-metal`)
still set `WWN_IOMFB_APPLE=1` or raw-`dlopen` Apple IOMFB.
