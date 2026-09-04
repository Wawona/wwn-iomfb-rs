# ABI (guest iOS 26.1)

Our findings. Authority binary: live vphone `wawona-jb` (iPhone99,11, 23B85)
plus the same-class research IPSW extract used for disassembly. Host macOS
DSC is not this table.

Ghidra program: `/IOMobileFramebuffer` in project
`GhidraVibe/ghidra-vibe-projects/wwn-iomfb` (Semeru 21, path must not
contain a `.name` segment). Confirmation for the swap family is that
program plus `ipsw dyld disass` of the named image extracted from the
iPhone99,11 23B85 cache. Host macOS DSC is not this table.

Status per symbol: `unconfirmed` | `confirmed` | `refuted` | `absent`.

## Program

| Field | Value |
|---|---|
| Guest | vphone `wawona-jb`, iPhone99,11, iOS 26.1 / 23B85 |
| Image | `IOMobileFramebuffer` |
| Extract | `~/.vphone/re/guest-iomfb/IOMobileFramebuffer` (176K, outside git) |
| Stock / class cache | research IPSW `iPhone99,11` 23B85 |
| Ghidra project | `/Users/8amps/GhidraVibe/ghidra-vibe-projects/wwn-iomfb` |
| Ghidra program | `/IOMobileFramebuffer` |
| Live guest IP | `192.168.64.104` (drifts; window title / `guest-ip.txt`) |
| Public exports | 153 `IOMobileFramebuffer*` T symbols |
| Tipa proof | `com.aspauldingcode.wawona.iomfb.smoke` in containers. Bind **153/153**. `GetMain` 1290x2796. `SwapSetLayer`/`SwapEnd` returned 0. `SwapWait(0)` returned `0xe000002b`. Restore ran. Sock shot was a full-frame present. |
| CFW note | Live guest DSC (5.5G, 82 slices) extracted. Live vs class Mach-O hashes differ (extract slide). Live `_kern_SwapEnd` is still `mov w3,#0x560` / sel 5. No CFW size patch on this guest. |

## Settled conflicts (S6)

| Conflict | Guest 26.1 answer | Status |
|---|---|---|
| SetLayer arity | 6-arg. `fb`, layer, IOSurface, src `CGRect` by value (d0-d3), dst `CGRect` (d4-d7), flags. 3-arg legacy is dead | confirmed |
| Layer count | 4. `_kern_SwapSetLayer` does `cmp w1, #4; b.lo`. Wawona's 3 is a conservative subset, not the ABI max | confirmed |
| SwapEnd public vs kern | Public `SwapEnd(fb)` is a 1-arg trampoline. `_kern_SwapEnd` is userclient method 5, struct size **0x560** | confirmed |
| GetDisplaySize type | Userspace out-param is `CGSize` (two doubles). Kernel selector 8 returns `{u32,u32}` then `ucvtf` | confirmed |
| Power-save polarity | `(fb, int)`, selector `0x21`. `0` disables savings (full power). No invert | confirmed |
| SwapCancel selector | `0x34` (decimal 52). Both notes were the same number | confirmed |
| Exclusive export | No export disables other clients. `SwapCancelAll` is this connection only. Hold is TrollStore policy | confirmed |

## Public trampoline shape (S5)

Confirmed on every sampled `Swap*` / size / power wrapper:

`cbz x0; ldr xN, [x0, #slot]; cbz xN; braaz xN` else `0xe00002c2`.

## Export census

| Symbol | Arity | CGRect | Notes | Status | Evidence |
|---|---|---|---|---|---|
| IOMobileFramebufferAnnounceNextSwapTimestamp | | | | unconfirmed | census |
| IOMobileFramebufferCalibrationBegin | | | | unconfirmed | census |
| IOMobileFramebufferCalibrationToolboxCommand | | | | unconfirmed | census |
| IOMobileFramebufferChangeFrameInfo | | | | unconfirmed | census |
| IOMobileFramebufferCopyLayerDisplayedSurface | | | | unconfirmed | census |
| IOMobileFramebufferCopyProperty | 2 (fb, key) | n/a | IORegistry copy; NULL on fail | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferCreateDisplayList | | | | unconfirmed | census |
| IOMobileFramebufferCreateStatistics | | | | unconfirmed | census |
| IOMobileFramebufferDisableCRCNotifications | 1 (fb) | n/a | notify type 4, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisableHotPlugDetectNotifications | 1 (fb) | n/a | notify type 0, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisableNeedSwapNotifications | 1 (fb) | n/a | notify type 6, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisablePowerNotifications | 1 (fb) | n/a | notify type 1, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisableVSyncNotifications | 1 (fb) | n/a | notify type 5, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableCRCNotifications | 3 (fb, cb, refcon) | n/a | notify type 4, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableDisableDithering | | | | unconfirmed | census |
| IOMobileFramebufferEnableDisableVideoPowerSavings | 2 (fb, int) | n/a | sel 0x21; 0 disables savings | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableHotPlugDetectNotifications | 3 (fb, cb, refcon) | n/a | notify type 0, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableMirroring | | | | unconfirmed | census |
| IOMobileFramebufferEnableNeedSwapNotifications | 3 (fb, cb, refcon) | n/a | notify type 6, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnablePowerNotifications | 3 (fb, cb, refcon) | n/a | notify type 1, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableStatistics | | | | unconfirmed | census |
| IOMobileFramebufferEnableVSyncNotifications | 3 (fb, cb, refcon) | n/a | notify type 5, sel 0x48. Not SetVsyncNotifications | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferFactoryPortal | | | | unconfirmed | census |
| IOMobileFramebufferFrameInfo | | | | unconfirmed | census |
| IOMobileFramebufferGetBandwidth | | | | unconfirmed | census |
| IOMobileFramebufferGetBlock | | | | unconfirmed | census |
| IOMobileFramebufferGetBrightnessControlCapabilities | | | | unconfirmed | census |
| IOMobileFramebufferGetBrightnessControlInfo | | | | unconfirmed | census |
| IOMobileFramebufferGetBufBlock | | | | unconfirmed | census |
| IOMobileFramebufferGetCRCNotifyMessageCount | | | | unconfirmed | census |
| IOMobileFramebufferGetCRCRunLoopSource | | | | unconfirmed | census |
| IOMobileFramebufferGetCanvasSizes | | | | unconfirmed | census |
| IOMobileFramebufferGetColorRemapMode | | | | unconfirmed | census |
| IOMobileFramebufferGetCurrentAbsoluteTime | | | | unconfirmed | census |
| IOMobileFramebufferGetDigitalOutMode | | | | unconfirmed | census |
| IOMobileFramebufferGetDigitalOutState | | | | unconfirmed | census |
| IOMobileFramebufferGetDisplayArea | 2 (fb, out) | n/a | sel 0x1d struct 8 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDisplaySize | 2 (fb, CGSize*) | n/a | sel 8 {u32,u32} then ucvtf to doubles | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDotPitch | 2 (fb, u32*) | n/a | sel 0x1c | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDotPitchFloat | 2 (fb, float*) | n/a | sel 0x5a | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetFrameworkInfo | 1 (out*) | n/a | stub returns 0xe00002f0 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetGammaTable | | | | unconfirmed | census |
| IOMobileFramebufferGetHDCPAuthenticationProtocol | | | | unconfirmed | census |
| IOMobileFramebufferGetHDCPDownstreamState | | | | unconfirmed | census |
| IOMobileFramebufferGetHDCPRunLoopSource | | | | unconfirmed | census |
| IOMobileFramebufferGetHotPlugRunLoopSource | | | | unconfirmed | census |
| IOMobileFramebufferGetID | 2 (fb, u32*) | n/a | sel 7 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetLayerDefaultSurface | 3 (fb, layer, IOSurface*) | n/a | sel 3 trampoline | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetLinkQuality | | | | unconfirmed | census |
| IOMobileFramebufferGetMainDisplay | 1 (fb**) | n/a | userspace walk + open | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetMatrix | | | | unconfirmed | census |
| IOMobileFramebufferGetMirrorError | | | | unconfirmed | census |
| IOMobileFramebufferGetProtectionOptions | | | | unconfirmed | census |
| IOMobileFramebufferGetPulseWidthMaximization | | | | unconfirmed | census |
| IOMobileFramebufferGetRunLoopSource | 2 (fb, type 0-7) | n/a | type 5 is vsync | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetSecondaryDisplay | 1 (fb**) | n/a | userspace walk + open | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetServiceObject | 1 (fb) | n/a | returns service at fb+0x10 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetSupportedDigitalOutModes | | | | unconfirmed | census |
| IOMobileFramebufferGetTypeID | 0 | n/a | CFTypeID via dispatch_once | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetVSyncRunLoopSource | 1 (fb) | n/a | notify type 5 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetWirelessSurface | | | | unconfirmed | census |
| IOMobileFramebufferGetWirelessSurfaceWithOptions | | | | unconfirmed | census |
| IOMobileFramebufferHDCPGetReply | | | | unconfirmed | census |
| IOMobileFramebufferHDCPSendRequest | | | | unconfirmed | census |
| IOMobileFramebufferInstallVirtualDisplay | | | | unconfirmed | census |
| IOMobileFramebufferInstallVirtualDisplays | | | | unconfirmed | census |
| IOMobileFramebufferIsMainDisplay | 2 (fb, u32*) | n/a | sel 0x12 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferKernelTests | | | | unconfirmed | census |
| IOMobileFramebufferOpen | 4 (service, task, type, fb**) | n/a | IOServiceOpen; conn at fb+0x14 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferOpenByName | 2 (CFString, fb**) | n/a | walks display infos; virt if type==2 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferReadyForSwap | 3 GPR | n/a | local; calls IsMainDisplay; not exclusive | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferRelbufInfo | | | | unconfirmed | census |
| IOMobileFramebufferRequestPowerChange | 2 (fb, int) | n/a | sel 0xc; 1=on, 0=off | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSPLCGetBrightness | | | | unconfirmed | census |
| IOMobileFramebufferSPLCSetBrightness | | | | unconfirmed | census |
| IOMobileFramebufferScheduleWithDispatchQueue | 3 (fb, type, queue) | n/a | local slot table; type 3 uses port sel 3 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetAmmoliteStrength | | | | unconfirmed | census |
| IOMobileFramebufferSetBlock | | | | unconfirmed | census |
| IOMobileFramebufferSetBrightnessControlCallback | | | | unconfirmed | census |
| IOMobileFramebufferSetBrightnessCorrection | | | | unconfirmed | census |
| IOMobileFramebufferSetCanvasSize | | | | unconfirmed | census |
| IOMobileFramebufferSetClamshellState | | | | unconfirmed | census |
| IOMobileFramebufferSetColorRemapMode | | | | unconfirmed | census |
| IOMobileFramebufferSetContrast | | | | unconfirmed | census |
| IOMobileFramebufferSetDebugFlags | | | | unconfirmed | census |
| IOMobileFramebufferSetDigitalOutMode | | | | unconfirmed | census |
| IOMobileFramebufferSetDisplayDevice | | | | unconfirmed | census |
| IOMobileFramebufferSetDroppable | 2 (fb, int) | n/a | userspace only; strb at +0x365 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetFlags | | | | unconfirmed | census |
| IOMobileFramebufferSetGammaTable | | | | unconfirmed | census |
| IOMobileFramebufferSetIdleBuffer | | | | unconfirmed | census |
| IOMobileFramebufferSetIdleBufferEvent | | | | unconfirmed | census |
| IOMobileFramebufferSetLine21Data | | | | unconfirmed | census |
| IOMobileFramebufferSetMatrix | | | | unconfirmed | census |
| IOMobileFramebufferSetMirrorContentRegion | | | | unconfirmed | census |
| IOMobileFramebufferSetParameter | | | | unconfirmed | census |
| IOMobileFramebufferSetPreset | | | | unconfirmed | census |
| IOMobileFramebufferSetRenderingAngle | | | | unconfirmed | census |
| IOMobileFramebufferSetTVOutMode | | | | unconfirmed | census |
| IOMobileFramebufferSetTVOutSignalType | | | | unconfirmed | census |
| IOMobileFramebufferSetTwilightStrength | | | | unconfirmed | census |
| IOMobileFramebufferSetUnderrunColor | | | | unconfirmed | census |
| IOMobileFramebufferSetVideoDACGain | | | | unconfirmed | census |
| IOMobileFramebufferSetWSSInfo | | | | unconfirmed | census |
| IOMobileFramebufferSetWhiteOnBlackMode | | | | unconfirmed | census |
| IOMobileFramebufferSupportedFrameInfo | | | | unconfirmed | census |
| IOMobileFramebufferSurfaceIsReplaceable | 4 (fb, surf, unused, bool*) | n/a | sel 0x31 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapActiveRegion | | | | unconfirmed | census |
| IOMobileFramebufferSwapBegin | 2 (fb, int*) | n/a | sel 4 trampoline | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapCancel | 2 (fb, token) | n/a | sel 0x34 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapCancelAll | 1 (fb) | n/a | sel 0x51; this connection only | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapCancelAllGetCurrent | | | | unconfirmed | census |
| IOMobileFramebufferSwapDebugInfo | | | | unconfirmed | census |
| IOMobileFramebufferSwapDirtyRegion | | | | unconfirmed | census |
| IOMobileFramebufferSwapEnd | 1 (fb) | n/a | sel 5 struct 0x560 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapGetCurrent | 2 (fb, u32*) | n/a | sel 0x5b | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSecureLayer | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetAmbientLux | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetBackgroundColor | 1 GPR + 3 FPR (s0-s2) | n/a | mutates pending BGRA at +0x16c | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetBlit | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetBrightness | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetBrightnessLimit | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetColorMatrix | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetColorRemapMode | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetDisplayEdr | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetDisplayEdrHeadroom | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetEventSignal | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetEventSignalOnGlass | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetEventWait | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetGainMap | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetGammaTable | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetICCCurve | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetICCMatrix | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetIndicatorBrightness | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetIndicatorBrightnessLimit | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetLayer | 6 | two CGRect by value (d0-d7) | kern cmp layer < 4 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetLayerEDRCompensation | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetParams | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetPulseWidthMaximization | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetSecureAnimation | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetTimestamp | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetTimestamps | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetToneMapConfig | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetUISubRegion | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetVideoDestEdgeAlpha | | | | unconfirmed | census |
| IOMobileFramebufferSwapSignal | | | | unconfirmed | census |
| IOMobileFramebufferSwapSubtitleRegion | | | | unconfirmed | census |
| IOMobileFramebufferSwapUIEdgeBlendMode | | | | unconfirmed | census |
| IOMobileFramebufferSwapWait | 3 (fb, token, options) | n/a | sel 6, 3 scalars | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapWaitWithTimeout | | | | unconfirmed | census |
| IOMobileFramebufferSwapWorkaroundSettings | | | | unconfirmed | census |
| IOMobileFramebufferUnscheduleFromDispatchQueue | 3 (fb, type, queue) | n/a | local | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferWaitSurface | 1 public | n/a | vtable +0x9a8 with w2=1; token at +0xb0 | confirmed | guest-class 23B85 ipsw disass |

Add `_kern_*` / `_virt_*` as they are confirmed. They are not public
`dlsym` names. Census of public `IOMobileFramebuffer*` is the 100% floor.

## `_kern_*` / `_virt_*` census

Local (`nm -a`), not `dlsym`. Guest 23B85: **101** `_kern_*`, **27** `_virt_*`.
Public wrappers stay the trampoline onto these. `_virt_*` is the in-process
virtual-display path (no IOConnect). Do not ship force-kern.

Notify types stuffed into `_kern_EnableNotifications` / Disable (sel `0x48`):
HotPlug=0, Power=1, HDCP=2, CRC=4, VSync=5, NeedSwap=6. Wiki selector 9
as `setVSyncNotifications` is **refuted**.

## SwapArg `0x560`

`_kern_SwapBegin` does `bzero(fb+0x18, 0x560)` and stores the token at
`fb+0xb0` (swap-rel `+0x98`). `_kern_SwapEnd` submits that struct with
selector 5.

Proven swap-relative offsets: token `+0x98`, surface IDs `+0x9c` (4 x u32),
dest/src ints `+0xac`, layer mask `+0x14c`, bgColor `+0x154`, gamma index
`+0x158`. `SwapSet*` mutates this pending struct. Kernel submit is End.

## GPU present

IOSurface is the dma-buf. Metal wrap of that surface is zero-copy.
`Display::present_iosurface` / `GpuSwapchain` commit layer 0 after a
Metal queue wait. Wait `0xe000002b` is incomplete, not a failed present.
vphone ships Metal.framework. See `docs/GPU.md` and
`scripts/build-tipa-metal.sh`.

## Milestone map

| Milestone | Family | ABI rows |
|---|---|---|
| M0 | Bootstrap | repo + flake |
| M1 | Lab extract | Mach-O on disk, named `IOMobileFramebuffer` |
| M2 | Census + claims | 153 public symbols listed |
| M3 | Swap family | confirmed above |
| M4 | Power and vsync | polarity confirmed; EnableVSyncNotifications type 5 / sel 0x48 |
| M5 | Exclusive search | no disable-others export; SwapCancelAll is this connection |
| M6 | Safe Rust + C ABI | swap / display / restore / GPU queue / identity bound |
| M7 | Tipa proof | CPU smoke + Metal tipa on vphone |
| M8 | Remaining exports | 153 names listed; M4/M5/open/SwapArg confirmed; other families still filling |
| M9 | Hand-off | Wawona may switch Mode B present. Not implemented here |
