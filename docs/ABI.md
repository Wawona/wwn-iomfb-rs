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
| Live guest IP | `192.168.64.110` (drifts; window title / `guest-ip.txt`) |
| Public exports | 153 `IOMobileFramebuffer*` T symbols |
| Live codes | Every public name has a vphone return code in [`docs/LIVE.md`](LIVE.md). Two stubs (`SwapSignal`, `SwapSetUISubRegion`) are `absent`. `InstallVirtualDisplay(s)` SIGSEGV on a null vtable (no fake virt funcs). |
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
| IOMobileFramebufferAnnounceNextSwapTimestamp | 2 GPR | n/a | trampoline; none; slot `+0x900` via x2. No matching `_kern_*` name. Timestamp helper family (`ingest`/`egest`) is local, not IOConnect | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferCalibrationBegin | 1 GPR | n/a | real; 0x14; Main-only. `IsMainDisplay` then SetParameter slot `+0xb60` with `w1=#0x14`, `w2=1` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferCalibrationToolboxCommand | 6 GPR | n/a | local+real; mixed; Saves x0-x5. Main-only command switch (`0x1010005`/`0x1010006`/`0x1020001`…). NVMe EAN / factory path; some arms hit I... | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferChangeFrameInfo | 2 GPR | n/a | trampoline→real; 0x49; Public walks InfoKey then slot `+0xb70`. `_kern_ChangeFrameInfo` sel `0x49` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferCopyLayerDisplayedSurface | 3 GPR | n/a | trampoline→real; 0x53; slot `+0x898`. kern `cmp w1,#3; b.hi` (layer 0-3). IOSurface out | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferCopyProperty | 2 (fb, key) | n/a | IORegistry copy; NULL on fail | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferCreateDisplayList | 1 GPR | n/a | local; none; `iomfb_populate_all_display_infos` then CF array of `_s_displays`. No IOConnect | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferCreateStatistics | 2 GPR | n/a | trampoline→local; none; slot `+0xb68`. kern uses service at `fb+0x10` (IORegistry), not IOConnect. NULL on miss | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisableCRCNotifications | 1 (fb) | n/a | notify type 4, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisableHotPlugDetectNotifications | 1 (fb) | n/a | notify type 0, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisableNeedSwapNotifications | 1 (fb) | n/a | notify type 6, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisablePowerNotifications | 1 (fb) | n/a | notify type 1, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferDisableVSyncNotifications | 1 (fb) | n/a | notify type 5, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableCRCNotifications | 3 (fb, cb, refcon) | n/a | notify type 4, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableDisableDithering | 2 GPR | n/a | trampoline→real; 0x1e; slot `+0xae0` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableDisableVideoPowerSavings | 2 (fb, int) | n/a | sel 0x21; 0 disables savings | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableHotPlugDetectNotifications | 3 (fb, cb, refcon) | n/a | notify type 0, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableMirroring | 3 GPR | n/a | real; 0x29; `(fb, other, enable)`. Main-only. `IOConnect` sel `0x29` struct 4. Fail `0xe00002c2` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableNeedSwapNotifications | 3 (fb, cb, refcon) | n/a | notify type 6, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnablePowerNotifications | 3 (fb, cb, refcon) | n/a | notify type 1, sel 0x48 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableStatistics | 2 GPR | n/a | trampoline→real; 0xd; slot `+0xa40` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferEnableVSyncNotifications | 3 (fb, cb, refcon) | n/a | notify type 5, sel 0x48. Not SetVsyncNotifications | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferFactoryPortal | 2 GPR | n/a | trampoline→real; 0x4b; slot `+0xb80` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferFrameInfo | 4 GPR | n/a | local+real; via slot; `(fb, buf, dest, count)`. `cbz x1` / `cbz w3`. InfoKey then slot `+0x9f0` with `w1=#3` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetBandwidth | 2 GPR | n/a | real; 0x73; GetBlock-family slot `+0xb58`, `w1=#0x73`, struct `0x20`. Copies 0x20 bytes to out. Fail `0xe00002c7` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetBlock | 6 GPR | n/a | trampoline→real; 0xaa; slot `+0xb58`. Generic scalar/struct trap. Fail `0xe00002c7` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetBrightnessControlCapabilities | 1 GPR | n/a | local; none; `dispatch_once` load brightness dylib; `braaz` into it. NULL if `fb+0xdc1!=1` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetBrightnessControlInfo | 2 GPR | n/a | local; none; Copies 0x18 bytes from `fb+0xdc8`. Needs `fb+0xdc1==1`. Else `0xe00002c2` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetBufBlock | 6 GPR | n/a | trampoline→real; 0xaa; slot `+0xba0`. Same sel as Get/SetBlock. Fail `0xe00002c7` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetCRCNotifyMessageCount | 1 GPR | n/a | trampoline→local; none; slot `+0xaf8`. kern `ldr x0,[x0,#0x6f8]` → mach message count. Returns 0 if no slot | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetCRCRunLoopSource | 1 GPR | n/a | trampoline; n/a (type 4); slot `+0xaf0` (`GetRunLoopSource`), stuffs `w1=#4` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetCanvasSizes | 3 GPR | n/a | local; none; `(fb, sizes**, count*)`. Calls slot `+0x8a0`, then `ucvtf` into doubles at `fb+0x5d0` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetColorRemapMode | 2 GPR | n/a | trampoline→real; 0x39; slot `+0xa30` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetCurrentAbsoluteTime | 2 GPR | n/a | trampoline→real; 0x4d; slot `+0xb88`. kern `w1=#0x4d` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDigitalOutMode | 3 GPR | n/a | real; 0x40; `(fb, out, out)`. GetBlock slot `+0xb58`, `w1=#0x40`, struct `0x2c`. Fail `0xe00002c7` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDigitalOutState | 2 GPR | n/a | trampoline→real; 0x19; slot `+0xa80` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDisplayArea | 2 (fb, out) | n/a | sel 0x1d struct 8 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDisplaySize | 2 (fb, CGSize*) | n/a | sel 8 {u32,u32} then ucvtf to doubles | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDotPitch | 2 (fb, u32*) | n/a | sel 0x1c | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetDotPitchFloat | 2 (fb, float*) | n/a | sel 0x5a | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetFrameworkInfo | 1 (out*) | n/a | stub returns 0xe00002f0 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetGammaTable | 2 GPR | n/a | trampoline→real; 0x1b; slot `+0xa68`. kern copies `0xc0c` bytes | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetHDCPAuthenticationProtocol | 1 GPR | n/a | trampoline; none visible; slot `+0xa98` via x1. Returns 0 if empty. No `_kern_*` name | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetHDCPDownstreamState | 2 GPR | n/a | trampoline→real; 0x18; slot `+0xaa0` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetHDCPRunLoopSource | 1 GPR | n/a | trampoline; n/a (type 2); slot `+0xaf0`, stuffs `w1=#2` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetHotPlugRunLoopSource | 1 GPR | n/a | trampoline; n/a (type 0); slot `+0xaf0`, stuffs `w1=#0` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetID | 2 (fb, u32*) | n/a | sel 7 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetLayerDefaultSurface | 3 (fb, layer, IOSurface*) | n/a | sel 3 trampoline | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetLinkQuality | 2 GPR | n/a | trampoline→real; 0x50; slot `+0xb98`. Empty slot returns `0x80000000`, not `0xe00002c2` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetMainDisplay | 1 (fb**) | n/a | userspace walk + open | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetMatrix | 3 GPR | n/a | real; 0x36; `(fb, mode, out*)`. `IOConnect` sel `0x36`, struct size `0x48` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetMirrorError | 2 GPR | n/a | local; none; Writes 0 to out if `fb+0xda0` set. Else log + `0xe00002c2` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetProtectionOptions | 2 GPR | n/a | trampoline→real; 0x52; slot `+0xab8` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetPulseWidthMaximization | 2 GPR | n/a | local; none; Main-only property walk via slot `+0x8b0`. Fail `0xe00002f0` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetRunLoopSource | 2 (fb, type 0-7) | n/a | type 5 is vsync | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetSecondaryDisplay | 1 (fb**) | n/a | userspace walk + open | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetServiceObject | 1 (fb) | n/a | returns service at fb+0x10 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetSupportedDigitalOutModes | 3 GPR | n/a | trampoline→local; none; slot `+0xa90`. kern builds CF via IORegistry (`fb+0x10`), not a userclient sel | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetTypeID | 0 | n/a | CFTypeID via dispatch_once | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetVSyncRunLoopSource | 1 (fb) | n/a | notify type 5 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetWirelessSurface | 3 GPR + 2 FPR (d0-d1) | 3 GPR + 2 FPR (d0-d1) | local; none; Tail-calls `WithOptions` with `w3=0`. First calls slot `+0xbb0` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferGetWirelessSurfaceWithOptions | 4 GPR + 2 FPR (d0-d1) | 4 GPR + 2 FPR (d0-d1) | local; none; Main-only. Walks wireless surface table at `fb+0xc10` (0x18-stride, 0x180 cap) | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferHDCPGetReply | 3 GPR | n/a | trampoline→real; 0x30; slot `+0xab0`. virt-HDCP path may bounce to `GetHDCPDownstreamState` sel `0x18` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferHDCPSendRequest | 5 GPR | n/a | trampoline→real; 0x2f; slot `+0xaa8`. kern `cbz x1`/`cbz x3` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferInstallVirtualDisplay | 4 GPR | n/a | local; none; Copies 0xB0-byte vtable from x2 into `_s_virt_funcs`; ctx x3. x0/x1 unused this build | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferInstallVirtualDisplays | 3 GPR | n/a | local; none; `(funcs*, ?, count)`. `count<=0xc`. Copies same vtable from x0; stores count at `_s_virt` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferIsMainDisplay | 2 (fb, u32*) | n/a | sel 0x12 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferKernelTests | 2 GPR | n/a | real; 0x38; `(fb, IOMFBKernelTestsArguments*)`. Demands `args->n < 0x15`. `IOConnect` sel `0x38` struct `0x9c` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferOpen | 4 (service, task, type, fb**) | n/a | IOServiceOpen; conn at fb+0x14 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferOpenByName | 2 (CFString, fb**) | n/a | walks display infos; virt if type==2 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferReadyForSwap | 3 GPR | n/a | local; calls IsMainDisplay; not exclusive | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferRelbufInfo | 4 GPR | n/a | local+real; via slot; `(fb, buf, dest, count)`. Driver-prop gated. Enabled: slot `+0x9f0` `w1=#7`. Disabled: slot `+0x9f8`. Else `0xe0000... | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferRequestPowerChange | 2 (fb, int) | n/a | sel 0xc; 1=on, 0=off | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSPLCGetBrightness | 2 GPR | n/a | trampoline→real; 0x43; slot `+0xb48` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSPLCSetBrightness | 2 GPR | n/a | trampoline→real; 0x42; slot `+0xb40` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferScheduleWithDispatchQueue | 3 (fb, type, queue) | n/a | local slot table; type 3 uses port sel 3 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetAmmoliteStrength | 1 GPR + 1 FPR (s0) | 1 GPR + 1 FPR (s0) | mutator; none; Clamp s0 to [0,1], scale, `str` at `fb+0x416`. Sibling of `_kern_SwapSetCEStrength` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetBlock | 6 GPR | n/a | trampoline→real; 0xaa; slot `+0xb50`. Fail `0xe00002c7` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetBrightnessControlCallback | 3 GPR | n/a | local; none; `(fb, cb, refcon)` stored at `+0xde0/+0xde8`. Loads brightness dylib. Fail `0xe00002c7` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetBrightnessCorrection | 2 GPR | n/a | trampoline→real; 0x32; slot `+0xa38` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetCanvasSize | 1 GPR + 2 FPR (d0-d1) | 1 GPR + 2 FPR (d0-d1) | real; 0x35; `fcvtzu` CGSize, `IOConnect` sel `0x35` two u32 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetClamshellState | 2 GPR | n/a | trampoline→real; 0x59; slot `+0xa88` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetColorRemapMode | 2 GPR | n/a | trampoline→real; 0x33; slot `+0xa28` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetContrast | 1 GPR + 1 FPR (s0) | 1 GPR + 1 FPR (s0) | local+real; 0x11; Fetches gamma (`0xc0c`) via GetGammaTable slot, remaps, SetGammaTable sel `0x11` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetDebugFlags | 4 GPR | n/a | trampoline→real; 0xf; slot `+0xa48` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetDigitalOutMode | 3 GPR | n/a | trampoline→real; 0x17; slot `+0xa78`. kern packs two ints, sel `0x17` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetDisplayDevice | 2 GPR | n/a | trampoline→real; 0x16; slot `+0xa70`. kern `cmp w1,#4; b.ls` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetDroppable | 2 (fb, int) | n/a | userspace only; strb at +0x365 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetFlags | 4 GPR | n/a | trampoline→real; 0xe; slot `+0xa50` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetGammaTable | 2 GPR | n/a | trampoline→real; 0x11; slot `+0xa58`. struct `0xc0c` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetIdleBuffer | 3 GPR | n/a | trampoline→real; via SwapEnd; slot `+0x9e0`. kern `_setIdleBuffer` then `braaz` SwapEnd slot `+0x978` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetIdleBufferEvent | 6 GPR | n/a | trampoline→real; via SwapEnd; slot `+0x9e8`. Stores ports at `+0x2c0/+0x2d4/+0x68/+0x90` then SwapEnd | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetLine21Data | 2 GPR | n/a | trampoline→real; 0x23; slot `+0xb10` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetMatrix | 3 GPR | n/a | real; 0x37; `(fb, mode, in*)`. `IOConnect` sel `0x37`, struct `0x48` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetMirrorContentRegion | 1 GPR + 4 FPR (d0-d3) | 1 GPR + 4 FPR (d0-d3) | real; 0x2c; CGRect by value → 4 floats, `IOConnect` sel `0x2c` struct `0x10` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetParameter | 4 GPR | n/a | trampoline→real; 0x44; slot `+0xb60` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetPreset | 2 GPR | n/a | trampoline→real; 0x3d; Remaps x1→x2, stuffs `w1=#0x3d`, `w3=#0x30`, `braaz` SetBlock slot `+0xb50`. Fail `0xe00002c7` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetRenderingAngle | 2 GPR | n/a | real; 0x2b; `(fb, float*)`. Main-only. Must be multiple of 90. `IOConnect` sel `0x2b` struct 4 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetTVOutMode | 2 GPR | n/a | trampoline→real; 0xa; slot `+0xa00` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetTVOutSignalType | 2 GPR | n/a | trampoline→real; 0x10; slot `+0xa08` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetTwilightStrength | 1 GPR + 1 FPR (s0) | 1 GPR + 1 FPR (s0) | mutator; none; Clamp s0 to [0,1], `str` at `fb+0x412`. Sibling of `_kern_SwapSetGCPStrength` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetUnderrunColor | 2 GPR | n/a | trampoline→real; 0x1f; slot `+0xae8` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetVideoDACGain | 1 GPR + 1 FPR (s0) | 1 GPR + 1 FPR (s0) | trampoline→real; 0x22; slot `+0xb08` via x1. kern `fcvtzu` s0, sel `0x22` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetWSSInfo | 3 GPR | n/a | trampoline→real; 0xb; slot `+0xa10` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSetWhiteOnBlackMode | 2 GPR | n/a | trampoline→real; 0x13; slot `+0xa20` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSupportedFrameInfo | 1 GPR | n/a | local; 0x4a (kern); Returns CF at `fb+0x7f0` after `InfoKeyInitialize`. NULL if slot `+0xb78` empty. kern helper uses sel `0x4a` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSurfaceIsReplaceable | 4 (fb, surf, unused, bool*) | n/a | sel 0x31 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapActiveRegion | 2 GPR + 4 FPR (d0-d3) | 2 GPR + 4 FPR (d0-d3) | mutator; none; `(fb, layer<4, CGRect)`. Writes rect list at `fb+0x1a0/+0x1b0`. Layer type 4 → `0xe00002e8` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapBegin | 2 (fb, int*) | n/a | sel 4 trampoline | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapCancel | 2 (fb, token) | n/a | sel 0x34 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapCancelAll | 1 (fb) | n/a | sel 0x51; this connection only | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapCancelAllGetCurrent | 2 GPR | n/a | trampoline→real; 0x5c; slot `+0x9d0` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapDebugInfo | 3 GPR | n/a | trampoline→mutator; none; slot `+0x8e8`. kern `stp x1,x2,[x0,#0x58]`; returns 0 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapDirtyRegion | 1 GPR + 4 FPR (d0-d3) | 1 GPR + 4 FPR (d0-d3) | mutator; none; CGRect → ints at `fb+0x2b0..+0x2bc`, flag `+0x2e4` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapEnd | 1 (fb) | n/a | sel 5 struct 0x560 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapGetCurrent | 2 (fb, u32*) | n/a | sel 0x5b | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSecureLayer | 4 GPR + 4 FPR (d0-d3) | 4 GPR + 4 FPR (d0-d3) | trampoline→mutator; none; slot `+0x8c0`. kern writes `+0x560..+0x574` and bit at `+0x367` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetAmbientLux | 1 GPR + 1 FPR (s0) | 1 GPR + 1 FPR (s0) | trampoline→mutator; none; slot `+0x958` via x1. kern `strb` `+0x41a`, `str s0` at `+0x41e`. Fail `0xe00002c7` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetBackgroundColor | 1 GPR + 3 FPR (s0-s2) | n/a | mutates pending BGRA at +0x16c | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetBlit | 3 GPR + 8 FPR (d0-d7) | 3 GPR + 8 FPR (d0-d7) | trampoline→mutator; none; Naked `ldr x3,[x0,#0x8e0]; braaz`. kern string: `(fb, IOSurface, CGRect, CGRect, uint32_t)`. Max 6 blits | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetBrightness | 2 GPR + 1 FPR (d0) | 2 GPR + 1 FPR (d0) | trampoline→mutator; none; slot `+0x918`. kern stores d0 at `+0x376`, flag `+0x36c`. Needs BC enable | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetBrightnessLimit | 2 GPR + 1 FPR (d0) | 2 GPR + 1 FPR (d0) | trampoline→mutator; none; slot `+0x938`. flag `+0x371`, value `+0x38e` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetColorMatrix | 4 GPR | n/a | trampoline→real; 0x45; slot `+0x980`. kern also sets `+0x34f`, then sel `0x45` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetColorRemapMode | 2 GPR | n/a | trampoline; none visible; slot `+0x988`. No `_kern_SwapSetColorRemapMode`. Pending-SwapArg style slot | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetDisplayEdr | 2 GPR + 1 FPR (d0) | 2 GPR + 1 FPR (d0) | trampoline→mutator; none; slot `+0x940`. flag `+0x372`, value `+0x396` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetDisplayEdrHeadroom | 1 GPR + 1 FPR (d0) | 1 GPR + 1 FPR (d0) | trampoline→mutator; none; slot `+0x948` via x1. flag `+0x374`, value `+0x3fe` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetEventSignal | 4 GPR | n/a | trampoline→mutator; none; slot `+0x8d8`. kern IOSurfaceID at `+0x2d4+4*idx`, ptr at `+0x90` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetEventSignalOnGlass | 3 GPR | n/a | trampoline→mutator; none; slot `+0x8d0`. stores `+0x2d0` and `+0x88` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetEventWait | 4 GPR | n/a | trampoline→mutator; none; slot `+0x8c8`. stores `+0x2c0+4*idx` and `+0x68` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetGainMap | 2 GPR | n/a | mutator; none; Releases prior map at `+0xd88`. Writes id at `+0x406`, flag `+0x40a` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetGammaTable | 2 GPR | n/a | mutator; none; `cmp w1,#2; b.hi` then `str w1,[fb,#0x170]` (gamma index 0-2) | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetICCCurve | 5 GPR | n/a | real; 0x33; `(fb, w1, x2, x3, struct*)`. Builds 0x290 blob, SetBlock slot `+0xb50` with `w1=#0x33` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetICCMatrix | 4 GPR | n/a | real; 0x32; `(fb, w1, w2, matrix*)`. SetBlock slot, `w1=#0x32`, struct `0x40` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetIndicatorBrightness | 2 GPR + 1 FPR (d0) | 2 GPR + 1 FPR (d0) | trampoline→mutator; none; Not the brightness slot. Loads `+0x918` then `braaz` `+0x920`. kern uses x1 + d0; flags `+0x36e/+0x36d`, value ... | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetIndicatorBrightnessLimit | 1 GPR + 1 FPR (d0) | 1 GPR + 1 FPR (d0) | trampoline→mutator; none; slot `+0x930` via x1. flag `+0x370`, value `+0x386` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetLayer | 6 | two CGRect by value (d0-d7) | kern cmp layer < 4 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetLayerEDRCompensation | 3 GPR + 1 FPR (d0) | 3 GPR + 1 FPR (d0) | trampoline→mutator; none; slot `+0x998`. kern `(fb, layer<4, bool, double)` into `+0x53c/+0x544` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetParams | 4 GPR | n/a | trampoline→mutator; none; Public `cmp w3,#4; b.hi` then slot `+0x910`. kern opcode switch 0-4 writes `+0x53b` etc | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetPulseWidthMaximization | 2 GPR | n/a | trampoline→mutator; none; slot `+0x990`. No IOConnect in kern | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetSecureAnimation | 2 GPR | n/a | trampoline→mutator; none; slot `+0x928`. kern `strb` at `+0x36f` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetTimestamp | 2 GPR | n/a | trampoline; none; slot `+0x8f8`. Timestamp ingest family; no IOConnect | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetTimestamps | 4 GPR | n/a | trampoline; none; slot `+0x908` via x4 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetToneMapConfig | 3 GPR | n/a | real; 0x6b; `(fb, struct*, flags)`. SetBlock slot `+0xb50`, `w1=#0x6b`, struct `0x4c` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetUISubRegion | n/a | n/a | stub; Immediate `mov w0,#0x2c7; movk #0xe000; ret`. Args unused | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSetVideoDestEdgeAlpha | 3 GPR | n/a | mutator; none; `stp x1,x2,[fb,#0x184]`. Returns 0 | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSignal | 1 GPR | n/a | stub; Public never `braaz`s. Checks slot `+0x9a0` and returns `0xe00002c2` or `0xe00002c7`. `_kern_SwapSignal` exists (sel `0x14`) but is unreachable from this export | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapSubtitleRegion | 3 GPR + 4 FPR (d0-d3) | 3 GPR + 4 FPR (d0-d3) | trampoline→mutator; none; slot `+0x8f0`. kern packs CGRect into 0x15-byte slots at `+0x2f9` (max 5) | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapUIEdgeBlendMode | 3 GPR + 1 FPR (s0) | 3 GPR + 1 FPR (s0) | mutator; none; `(fb, layer<=1, mode, s0 in [-1,0])`. Writes `+0x104/+0x114` | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapWait | 3 (fb, token, options) | n/a | sel 6, 3 scalars | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapWaitWithTimeout | 3 GPR + 1 FPR (d0) | 3 GPR + 1 FPR (d0) | trampoline→real; 0x6; slot `+0x9b0`. Same sel as `SwapWait`; kern multiplies d0 by a constant, 3 scalars | confirmed | guest-class 23B85 ipsw disass |
| IOMobileFramebufferSwapWorkaroundSettings | 3 GPR | n/a | trampoline→mutator; none; slot `+0xb90` → `_kern_SwapWARSettings`. Writes `+0x351` and 16 bytes at `+0x355` | confirmed | guest-class 23B85 ipsw disass |
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

Proven swap-relative offsets (object `fb+0x18+off` unless noted as `fb+`):

| Off | Field | Writer |
|---|---|---|
| `+0x98` | token | SwapBegin (`fb+0xb0`) |
| `+0x9c` | surface IDs (4 x u32) | SwapSetLayer |
| `+0xac` | dest/src ints | SwapSetLayer |
| `+0x14c` | layer mask | SwapSetLayer |
| `+0x154` | bgColor | SwapSetBackgroundColor |
| `+0x158` | gamma index | SwapSetGammaTable (`fb+0x170`) |
| `fb+0x184` | video dest edge alpha | SwapSetVideoDestEdgeAlpha |
| `fb+0x1a0` / `+0x1b0` | active region | SwapActiveRegion |
| `fb+0x2b0` | dirty region | SwapDirtyRegion |
| `fb+0x2c0` / `+0x68` | event wait | SwapSetEventWait |
| `fb+0x2d0` / `+0x88` | event signal on glass | SwapSetEventSignalOnGlass |
| `fb+0x2d4` / `+0x90` | event signal | SwapSetEventSignal |
| `fb+0x2f9` | subtitle slots | SwapSubtitleRegion |
| `fb+0x34f` | color matrix flag | SwapSetColorMatrix |
| `fb+0x351` / `+0x355` | WAR settings | SwapWorkaroundSettings |
| `fb+0x36c` / `+0x376` | brightness | SwapSetBrightness |
| `fb+0x406` / `+0x40a` | gain map | SwapSetGainMap |
| `fb+0x412` / `+0x416` | twilight / ammolite | SetTwilightStrength / SetAmmoliteStrength |
| `fb+0x53c` / `+0x544` | layer EDR | SwapSetLayerEDRCompensation |

`SwapSet*` mutates this pending struct. Kernel submit is End. Public
`SwapSignal` and `SwapSetUISubRegion` are stubs.

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
| M6 | Safe Rust + C ABI | All 153 confirmed names callable (`iomfb_export` / typed families) |
| M7 | Tipa proof | CPU smoke + Metal tipa on vphone |
| M8 | Remaining exports | 153/153 confirmed and live-called. Two stubs: `SwapSignal`, `SwapSetUISubRegion`. Codes: `docs/LIVE.md` |
| M9 | Hand-off | Wawona may switch Mode B present. Not implemented here |
