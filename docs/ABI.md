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
| Power-save polarity | Call is `(fb, int)`, selector `0x21`. Whether `0` disables savings is still unconfirmed | unconfirmed |
| SwapCancel selector | `0x34` (decimal 52). Both notes were the same number | confirmed |
| Exclusive export | No export name disables other clients. `SwapCancelAll` exists; hold remains the TrollStore model until proven otherwise | unconfirmed |

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
| IOMobileFramebufferCopyProperty | | | | unconfirmed | census |
| IOMobileFramebufferCreateDisplayList | | | | unconfirmed | census |
| IOMobileFramebufferCreateStatistics | | | | unconfirmed | census |
| IOMobileFramebufferDisableCRCNotifications | | | | unconfirmed | census |
| IOMobileFramebufferDisableHotPlugDetectNotifications | | | | unconfirmed | census |
| IOMobileFramebufferDisableNeedSwapNotifications | | | | unconfirmed | census |
| IOMobileFramebufferDisablePowerNotifications | | | | unconfirmed | census |
| IOMobileFramebufferDisableVSyncNotifications | | | | unconfirmed | census |
| IOMobileFramebufferEnableCRCNotifications | | | | unconfirmed | census |
| IOMobileFramebufferEnableDisableDithering | | | | unconfirmed | census |
| IOMobileFramebufferEnableDisableVideoPowerSavings | 2 (fb, int) | n/a | sel 0x21; polarity unconfirmed | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableHotPlugDetectNotifications | | | | unconfirmed | census |
| IOMobileFramebufferEnableMirroring | | | | unconfirmed | census |
| IOMobileFramebufferEnableNeedSwapNotifications | | | | unconfirmed | census |
| IOMobileFramebufferEnablePowerNotifications | | | | unconfirmed | census |
| IOMobileFramebufferEnableStatistics | | | | unconfirmed | census |
| IOMobileFramebufferEnableVSyncNotifications | | | | unconfirmed | census |
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
| IOMobileFramebufferGetDisplayArea | | | | unconfirmed | census |
| IOMobileFramebufferGetDisplaySize | 2 (fb, CGSize*) | n/a | sel 8 {u32,u32} then ucvtf to doubles | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDotPitch | | | | unconfirmed | census |
| IOMobileFramebufferGetDotPitchFloat | | | | unconfirmed | census |
| IOMobileFramebufferGetFrameworkInfo | | | | unconfirmed | census |
| IOMobileFramebufferGetGammaTable | | | | unconfirmed | census |
| IOMobileFramebufferGetHDCPAuthenticationProtocol | | | | unconfirmed | census |
| IOMobileFramebufferGetHDCPDownstreamState | | | | unconfirmed | census |
| IOMobileFramebufferGetHDCPRunLoopSource | | | | unconfirmed | census |
| IOMobileFramebufferGetHotPlugRunLoopSource | | | | unconfirmed | census |
| IOMobileFramebufferGetID | | | | unconfirmed | census |
| IOMobileFramebufferGetLayerDefaultSurface | 3 (fb, layer, IOSurface*) | n/a | sel 3 trampoline | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetLinkQuality | | | | unconfirmed | census |
| IOMobileFramebufferGetMainDisplay | 1 (fb**) | n/a | userspace walk + open | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetMatrix | | | | unconfirmed | census |
| IOMobileFramebufferGetMirrorError | | | | unconfirmed | census |
| IOMobileFramebufferGetProtectionOptions | | | | unconfirmed | census |
| IOMobileFramebufferGetPulseWidthMaximization | | | | unconfirmed | census |
| IOMobileFramebufferGetRunLoopSource | | | | unconfirmed | census |
| IOMobileFramebufferGetSecondaryDisplay | 1 (fb**) | n/a | userspace walk + open | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetServiceObject | | | | unconfirmed | census |
| IOMobileFramebufferGetSupportedDigitalOutModes | | | | unconfirmed | census |
| IOMobileFramebufferGetTypeID | | | | unconfirmed | census |
| IOMobileFramebufferGetVSyncRunLoopSource | | | | unconfirmed | census |
| IOMobileFramebufferGetWirelessSurface | | | | unconfirmed | census |
| IOMobileFramebufferGetWirelessSurfaceWithOptions | | | | unconfirmed | census |
| IOMobileFramebufferHDCPGetReply | | | | unconfirmed | census |
| IOMobileFramebufferHDCPSendRequest | | | | unconfirmed | census |
| IOMobileFramebufferInstallVirtualDisplay | | | | unconfirmed | census |
| IOMobileFramebufferInstallVirtualDisplays | | | | unconfirmed | census |
| IOMobileFramebufferIsMainDisplay | | | | unconfirmed | census |
| IOMobileFramebufferKernelTests | | | | unconfirmed | census |
| IOMobileFramebufferOpen | | | | unconfirmed | census |
| IOMobileFramebufferOpenByName | | | | unconfirmed | census |
| IOMobileFramebufferReadyForSwap | | | | unconfirmed | census |
| IOMobileFramebufferRelbufInfo | | | | unconfirmed | census |
| IOMobileFramebufferRequestPowerChange | 2 (fb, int) | n/a | kern; 1=on lead | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSPLCGetBrightness | | | | unconfirmed | census |
| IOMobileFramebufferSPLCSetBrightness | | | | unconfirmed | census |
| IOMobileFramebufferScheduleWithDispatchQueue | | | | unconfirmed | census |
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
| IOMobileFramebufferSetDroppable | | | | unconfirmed | census |
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
| IOMobileFramebufferSurfaceIsReplaceable | | | | unconfirmed | census |
| IOMobileFramebufferSwapActiveRegion | | | | unconfirmed | census |
| IOMobileFramebufferSwapBegin | 2 (fb, int*) | n/a | sel 4 trampoline | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapCancel | 2 (fb, token) | n/a | sel 0x34 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapCancelAll | | | | unconfirmed | census |
| IOMobileFramebufferSwapCancelAllGetCurrent | | | | unconfirmed | census |
| IOMobileFramebufferSwapDebugInfo | | | | unconfirmed | census |
| IOMobileFramebufferSwapDirtyRegion | | | | unconfirmed | census |
| IOMobileFramebufferSwapEnd | 1 (fb) | n/a | sel 5 struct 0x560 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapGetCurrent | | | | unconfirmed | census |
| IOMobileFramebufferSwapSecureLayer | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetAmbientLux | | | | unconfirmed | census |
| IOMobileFramebufferSwapSetBackgroundColor | | | | unconfirmed | census |
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
| IOMobileFramebufferUnscheduleFromDispatchQueue | | | | unconfirmed | census |
| IOMobileFramebufferWaitSurface | | | | unconfirmed | census |

Add `_kern_*` / `_virt_*` as they are confirmed. They are not public
`dlsym` names. Census of public `IOMobileFramebuffer*` is the 100% floor.

## GPU present

IOSurface is the dma-buf. Metal wrap of that surface is zero-copy.
`Display::present_iosurface` / `GpuSwapchain` commit layer 0. Wait
`0xe000002b` is incomplete, not a failed present. See `docs/GPU.md`.

## Milestone map

| Milestone | Family | ABI rows |
|---|---|---|
| M0 | Bootstrap | repo + flake |
| M1 | Lab extract | Mach-O on disk, named `IOMobileFramebuffer` |
| M2 | Census + claims | 153 public symbols listed |
| M3 | Swap family | confirmed above |
| M4 | Power and vsync | power wrappers bound; polarity / vsync still open |
| M5 | Exclusive search | no obvious disable-others export |
| M6 | Safe Rust + C ABI | swap / display / restore implemented |
| M7 | Tipa proof | containers install; present set=0 end=0; restore; bind census |
| M8 | Remaining exports | 153 names in `PUBLIC_EXPORTS`; tipa `bind N/153`; arity open except swap family |
| M9 | Hand-off | Wawona may switch Mode B present. Not implemented here |
