# Userspace IOMFB (product path)

The crate implements the confirmed IOMFB *ABI* in process memory.
It does not open `IOMobileFramebufferUserClient` and does not call
`IOConnect`.

```text
Metal / compositor
  -> IOSurface (dma-buf)
  -> Display::present_iosurface
  -> optional present callback (CALayer, iland, CAMetalLayer)
```

Apple `dlopen` is a lab oracle only. Set `WWN_IOMFB_APPLE=1` for
`iomfb-live` / `iomfb-matrix`. Product callers use
`Display::userland(w, h)` or `iomfb_display_open_userland`.

`Display::main()` resolves size from, in order:

1. `iomfb_display_configure` / `configure_userland`
2. `WWN_IOMFB_WIDTH` / `WWN_IOMFB_HEIGHT`
3. Public `UIScreen.nativeBounds` (iOS family glue)

HDCP, factory, calibration, and `KernelTests` return `Error::Absent`
on the userspace backend. Those families exist to talk to hardware.

`InstallVirtualDisplay(s)` stays fail-closed. This crate does not
invent a virt vtable.

Wawona L4 Desktop still binds later (`S1-iland-bind`). The hook is
`iomfb_display_set_present` / `GpuSwapchain::present_external`.
