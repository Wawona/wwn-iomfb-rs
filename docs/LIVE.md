# Live vphone return codes

Guest: vphone `wawona-jb`, iPhone99,11, iOS 26.1 / 23B85.
Tool: `iomfb-live --one NAME` (crate-linked `libiomfb_c.a`).
Each public export is one process so a trap cannot abort the census.

A recorded row is a live call at the census arity. `0` is
`IOReturn` success. Negative values are Apple codes (or a
truncated pointer for CF / IOService getters). Two stubs are
`absent` and are not invoked. `InstallVirtualDisplay(s)` still
SIGSEGV on a null vtable. That is the fail-closed result. This
crate does not invent virt funcs.

Typed helpers (same guest, before the per-export loop):

| Helper | Result |
|---|---|
| `iomfb_bound_export_count` | 153/153 |
| display | 1290x2796, id=17 |
| `iomfb_get_type_id` | 73 |
| `iomfb_get_service_object` | non-null |
| `iomfb_ready_for_swap` | `0xE00002C2` |
| `iomfb_enable_vsync` / disable | 0 / 0 |
| `iomfb_get_gamma_table` / set | 0 / 0 |
| `iomfb_kernel_tests` (0x9c, n=0) | 0 |
| `iomfb_factory_portal(NULL)` | `0xE00002C2` |
| `iomfb_hdcp_send_request` null | `0xE00002C2` |
| `iomfb_open_by_name(primary)` | 0 |

## Public exports

| Symbol | Live | Notes |
|---|---|---|
| `IOMobileFramebufferAnnounceNextSwapTimestamp` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferCalibrationBegin` | `-536870170` (`0xE00002E6`) |  |
| `IOMobileFramebufferCalibrationToolboxCommand` | `-536870170` (`0xE00002E6`) |  |
| `IOMobileFramebufferChangeFrameInfo` | `0` |  |
| `IOMobileFramebufferCopyLayerDisplayedSurface` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferCopyProperty` | `0` | CFSTR(IOMFB) returned NULL |
| `IOMobileFramebufferCreateDisplayList` | `-1996258624` (`0x890382C0`) | CF array pointer truncated to i32 |
| `IOMobileFramebufferCreateStatistics` | `0` |  |
| `IOMobileFramebufferDisableCRCNotifications` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferDisableHotPlugDetectNotifications` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferDisableNeedSwapNotifications` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferDisablePowerNotifications` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferDisableVSyncNotifications` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferEnableCRCNotifications` | `0` |  |
| `IOMobileFramebufferEnableDisableDithering` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferEnableDisableVideoPowerSavings` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferEnableHotPlugDetectNotifications` | `0` |  |
| `IOMobileFramebufferEnableMirroring` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferEnableNeedSwapNotifications` | `0` |  |
| `IOMobileFramebufferEnablePowerNotifications` | `0` |  |
| `IOMobileFramebufferEnableStatistics` | `0` |  |
| `IOMobileFramebufferEnableVSyncNotifications` | `0` |  |
| `IOMobileFramebufferFactoryPortal` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferFrameInfo` | `0` |  |
| `IOMobileFramebufferGetBandwidth` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferGetBlock` | `0` |  |
| `IOMobileFramebufferGetBrightnessControlCapabilities` | `0` |  |
| `IOMobileFramebufferGetBrightnessControlInfo` | `-536870208` (`0xE00002C0`) |  |
| `IOMobileFramebufferGetBufBlock` | `0` |  |
| `IOMobileFramebufferGetCRCNotifyMessageCount` | `0` |  |
| `IOMobileFramebufferGetCRCRunLoopSource` | `1925333760` | CFRunLoopSource pointer truncated |
| `IOMobileFramebufferGetCanvasSizes` | `0` |  |
| `IOMobileFramebufferGetColorRemapMode` | `0` |  |
| `IOMobileFramebufferGetCurrentAbsoluteTime` | `0` |  |
| `IOMobileFramebufferGetDigitalOutMode` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferGetDigitalOutState` | `0` |  |
| `IOMobileFramebufferGetDisplayArea` | `0` |  |
| `IOMobileFramebufferGetDisplaySize` | `0` |  |
| `IOMobileFramebufferGetDotPitch` | `0` |  |
| `IOMobileFramebufferGetDotPitchFloat` | `0` |  |
| `IOMobileFramebufferGetFrameworkInfo` | `-536870160` (`0xE00002F0`) | confirmed stub `0xe00002f0` |
| `IOMobileFramebufferGetGammaTable` | `0` |  |
| `IOMobileFramebufferGetHDCPAuthenticationProtocol` | `1` |  |
| `IOMobileFramebufferGetHDCPDownstreamState` | `0` |  |
| `IOMobileFramebufferGetHDCPRunLoopSource` | `1522680576` | CFRunLoopSource pointer truncated |
| `IOMobileFramebufferGetHotPlugRunLoopSource` | `2126660352` | CFRunLoopSource pointer truncated |
| `IOMobileFramebufferGetID` | `0` |  |
| `IOMobileFramebufferGetLayerDefaultSurface` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferGetLinkQuality` | `-2147483648` (`0x80000000`) | empty slot `0x80000000` |
| `IOMobileFramebufferGetMainDisplay` | `0` |  |
| `IOMobileFramebufferGetMatrix` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferGetMirrorError` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferGetProtectionOptions` | `0` |  |
| `IOMobileFramebufferGetPulseWidthMaximization` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferGetRunLoopSource` | `0` |  |
| `IOMobileFramebufferGetSecondaryDisplay` | `-536870208` (`0xE00002C0`) |  |
| `IOMobileFramebufferGetServiceObject` | `14083` | IOService pointer truncated |
| `IOMobileFramebufferGetSupportedDigitalOutModes` | `0` |  |
| `IOMobileFramebufferGetTypeID` | `73` | CFTypeID |
| `IOMobileFramebufferGetVSyncRunLoopSource` | `12731136` | CFRunLoopSource pointer truncated |
| `IOMobileFramebufferGetWirelessSurface` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferGetWirelessSurfaceWithOptions` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferHDCPGetReply` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferHDCPSendRequest` | `-536870206` (`0xE00002C2`) | null req/reply |
| `IOMobileFramebufferInstallVirtualDisplay` | SIGSEGV (signal 11). Null vtable. No fake virt funcs | x2 NULL. Do not ship a fake vtable |
| `IOMobileFramebufferInstallVirtualDisplays` | SIGSEGV (signal 11). Null vtable. No fake virt funcs | x0 NULL. Do not ship a fake vtable |
| `IOMobileFramebufferIsMainDisplay` | `0` |  |
| `IOMobileFramebufferKernelTests` | `0` | zeroed `0x9c`, `n=0` |
| `IOMobileFramebufferOpen` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferOpenByName` | `-536870206` (`0xE00002C2`) | null CFString in live_call; typed CFSTR(primary) returned 0 |
| `IOMobileFramebufferReadyForSwap` | `-536870206` (`0xE00002C2`) | 3 GPR `(fb, NULL, 0)` |
| `IOMobileFramebufferRelbufInfo` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferRequestPowerChange` | `0` |  |
| `IOMobileFramebufferSPLCGetBrightness` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSPLCSetBrightness` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferScheduleWithDispatchQueue` | `0` |  |
| `IOMobileFramebufferSetAmmoliteStrength` | `0` |  |
| `IOMobileFramebufferSetBlock` | `0` |  |
| `IOMobileFramebufferSetBrightnessControlCallback` | `0` |  |
| `IOMobileFramebufferSetBrightnessCorrection` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetCanvasSize` | `0` |  |
| `IOMobileFramebufferSetClamshellState` | `0` |  |
| `IOMobileFramebufferSetColorRemapMode` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSetContrast` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetDebugFlags` | `0` |  |
| `IOMobileFramebufferSetDigitalOutMode` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetDisplayDevice` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSetDroppable` | `0` |  |
| `IOMobileFramebufferSetFlags` | `0` |  |
| `IOMobileFramebufferSetGammaTable` | `0` |  |
| `IOMobileFramebufferSetIdleBuffer` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSetIdleBufferEvent` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSetLine21Data` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetMatrix` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetMirrorContentRegion` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetParameter` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSetPreset` | `-536870207` (`0xE00002C1`) |  |
| `IOMobileFramebufferSetRenderingAngle` | `-536870170` (`0xE00002E6`) |  |
| `IOMobileFramebufferSetTVOutMode` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetTVOutSignalType` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetTwilightStrength` | `0` |  |
| `IOMobileFramebufferSetUnderrunColor` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetVideoDACGain` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetWSSInfo` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSetWhiteOnBlackMode` | `0` |  |
| `IOMobileFramebufferSupportedFrameInfo` | `0` |  |
| `IOMobileFramebufferSurfaceIsReplaceable` | `0` |  |
| `IOMobileFramebufferSwapActiveRegion` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapBegin` | `0` |  |
| `IOMobileFramebufferSwapCancel` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapCancelAll` | `0` |  |
| `IOMobileFramebufferSwapCancelAllGetCurrent` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSwapDebugInfo` | `0` |  |
| `IOMobileFramebufferSwapDirtyRegion` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapEnd` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferSwapGetCurrent` | `-536870201` (`0xE00002C7`) |  |
| `IOMobileFramebufferSwapSecureLayer` | `0` |  |
| `IOMobileFramebufferSwapSetAmbientLux` | `0` |  |
| `IOMobileFramebufferSwapSetBackgroundColor` | `0` |  |
| `IOMobileFramebufferSwapSetBlit` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapSetBrightness` | `0` |  |
| `IOMobileFramebufferSwapSetBrightnessLimit` | `0` |  |
| `IOMobileFramebufferSwapSetColorMatrix` | `0` |  |
| `IOMobileFramebufferSwapSetColorRemapMode` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapSetDisplayEdr` | `0` |  |
| `IOMobileFramebufferSwapSetDisplayEdrHeadroom` | `0` |  |
| `IOMobileFramebufferSwapSetEventSignal` | `0` |  |
| `IOMobileFramebufferSwapSetEventSignalOnGlass` | `0` |  |
| `IOMobileFramebufferSwapSetEventWait` | `0` |  |
| `IOMobileFramebufferSwapSetGainMap` | `0` |  |
| `IOMobileFramebufferSwapSetGammaTable` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapSetICCCurve` | `-536870207` (`0xE00002C1`) |  |
| `IOMobileFramebufferSwapSetICCMatrix` | `-536870207` (`0xE00002C1`) |  |
| `IOMobileFramebufferSwapSetIndicatorBrightness` | `0` |  |
| `IOMobileFramebufferSwapSetIndicatorBrightnessLimit` | `0` |  |
| `IOMobileFramebufferSwapSetLayer` | `0` |  |
| `IOMobileFramebufferSwapSetLayerEDRCompensation` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapSetParams` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapSetPulseWidthMaximization` | `0` |  |
| `IOMobileFramebufferSwapSetSecureAnimation` | `0` |  |
| `IOMobileFramebufferSwapSetTimestamp` | `0` |  |
| `IOMobileFramebufferSwapSetTimestamps` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapSetToneMapConfig` | `-536870207` (`0xE00002C1`) |  |
| `IOMobileFramebufferSwapSetUISubRegion` | `absent` (stub, not called) |  |
| `IOMobileFramebufferSwapSetVideoDestEdgeAlpha` | `0` |  |
| `IOMobileFramebufferSwapSignal` | `absent` (stub, not called) |  |
| `IOMobileFramebufferSwapSubtitleRegion` | `0` |  |
| `IOMobileFramebufferSwapUIEdgeBlendMode` | `-536870206` (`0xE00002C2`) |  |
| `IOMobileFramebufferSwapWait` | `0` |  |
| `IOMobileFramebufferSwapWaitWithTimeout` | `0` |  |
| `IOMobileFramebufferSwapWorkaroundSettings` | `0` |  |
| `IOMobileFramebufferUnscheduleFromDispatchQueue` | `-536870212` (`0xE00002BC`) |  |
| `IOMobileFramebufferWaitSurface` | `0` |  |
