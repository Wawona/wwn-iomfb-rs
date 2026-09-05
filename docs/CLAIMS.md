# Claims ingest

Every prior claim starts **unconfirmed**. A row becomes crate ABI only after
`docs/ABI.md` records a guest 26.1 Ghidra pass (and, for swap / power /
restore, a tipa or lab call that returns 0).

Status: `unconfirmed` | `confirmed` | `refuted` | `absent`.

Cite `docs/SOURCES.md`. Do not copy gist or wiki C.

## S1. Wawona Mode B (in use, unverified)

| Id | Claim | Source | Status |
|---|---|---|---|
| S1-dlopen | `dlopen` path `…/IOMobileFramebuffer.framework/IOMobileFramebuffer`. Framework dir is a stub; real code is DSC | wwn-m | confirmed |
| S1-main | `IOMobileFramebufferGetMainDisplay(IOMobileFramebufferRef*)` | wwn-m | confirmed |
| S1-secondary | `GetSecondaryDisplay` fallback if main fails | wwn-m | confirmed |
| S1-size-cgsize | `GetDisplaySize(fb, CGSize*)` | wwn-h | confirmed |
| S1-swap-begin | `SwapBegin(fb, int *token)` | wwn-m | confirmed |
| S1-swap-end-1arg | Public `SwapEnd(fb)` is 1-arg. Token only from Begin | wwn-m | confirmed |
| S1-swap-wait-0 | `SwapWait(fb, token, options)` with `options==0` meaning until displayed | wwn-m | confirmed |
| S1-setlayer-6 | `SwapSetLayer(fb, layer, IOSurface, src CGRect, dst CGRect, flags)` is 6-arg. Layer `0`, full-rect src/dst, flags `0` | wwn-m / screendump | confirmed |
| S1-default-surface | `GetLayerDefaultSurface(fb, layer, IOSurface*)`. Layer `0` is SpringBoard CA. Restore-only. Never a render target | wwn-m | confirmed |
| S1-power-save-0 | `EnableDisableVideoPowerSavings(fb, int)`. Open passes `0` (disable savings). Restore passes `1` | wwn-m | confirmed |
| S1-power-change-1 | `RequestPowerChange(fb, int)`. Open passes `1` (on) | wwn-m | confirmed |
| S1-layers-3 | `WWN_IOMFB_LAYER_COUNT == 3`. Present only our BGRA (`BGRA` / `ARGB`) IOSurfaces | wwn-h | refuted |
| S1-restore | Restore: Begin + SetLayer default (or NULL + zero rects) + End + Wait | wwn-m | confirmed |
| S1-hold | Exclusive hold is not an IOMFB export. Re-present last surface if no client swap for ~12 ms. `SwapWait` every commit | wwn-rs | confirmed |
| S1-sel | Userclient selectors 3 default, 4 begin, 5 end, 6 wait, 8 size, 9 vsync (notify type 5 / sel 0x48), 12 power, 52 cancel (per-token) | wiki / aiaf / wwn-h | confirmed |
| S1-cancel-absent | `SwapCancel` is not in the Wawona trampoline | wwn-m | confirmed |
| S1-iland-bind | Weston DRM page-flip -> `iomfb_display_set_present` / `present_external` (Wawona L4). Not this crate | wwn-present | unconfirmed |
| S1-ents | Tipa ents: `com.apple.private.IOMobileFramebuffer`, `IOMobileFramebufferUserClient` + `IOSurfaceRootUserClient`, `no-sandbox` / `platform-application`, `allow-explicit-graphics-priority`. Never IOWatchdog | fbvnc / Wawona tipa rule | confirmed |

HID park in `WWNModeBDisplayClaim.m` is **not** IOMFB. Out of this crate.

## S2. Public userspace headers (legacy vs modern)

Treat as pre-iOS-7 / iOS-7 unless Ghidra says they still exist on 26.1.

| Id | Claim | Source | Status |
|---|---|---|---|
| S2-open | `IOMobileFramebufferOpen(service, task, type, fb*)` same shape as `IOServiceOpen`, type `0` | gist-2015 / rms | confirmed |
| S2-open-name | `OpenByName` with `primary` / `external` / `wireless` | gist-2015 | confirmed |
| S2-getters | `GetMainDisplay`, `GetDisplaySize`, `GetDisplayArea`, `GetID`, `GetDotPitch`, `IsMainDisplay` | gist-2015 | confirmed |
| S2-swap-begin | `SwapBegin(fb, token*)` | gist-2015 | confirmed |
| S2-swap-end-1arg | `SwapEnd(fb)` 1-arg | gist-2015 | confirmed |
| S2-setlayer-3 | `SwapSetLayer` is 3-arg `(fb, layer, buffer)` with `CoreSurfaceBufferRef` or `IOSurfaceRef` | gist-2015 / rms | refuted |
| S2-swap-wait | `SwapWait(fb, token, something)` | gist-2015 | confirmed |
| S2-default | `GetLayerDefaultSurface(fb, layer, buffer*)`. Layer, not surfaceId | gist-nevyn | confirmed |
| S2-color | `Get/SetGammaTable` (0xc0c), `SetContrast`, `Get/SetColorRemapMode`, `SetWhiteOnBlackMode`, `SetBrightnessCorrection`, `Get/SetMatrix` | gist-2015 | confirmed |
| S2-power-yes | Export exists. Polarity is int `0`=disable savings, not Cocoa YES/NO | gist-2015 | confirmed |
| S2-types | IOReturn family `0xE000xxxx`. Gamma table `0xc0c`. Matrix IOConnect `0x48` bytes | gist-2015 | confirmed |
| S2-setlayer-6 | `SwapSetLayer(fb, layer, IOSurface, CGRect bounds, CGRect frame, int flags)` 6-arg | screendump | confirmed |

S2-setlayer-3 conflicts with S1-setlayer-6 / S2-setlayer-6. Guest 26.1 wins.

## S3. Apple Wiki userclient

Verify each selector **and** the `SwapArg` fields on guest 26.1.

| Id | Claim | Status |
|---|---|---|
| S3-sel-3 | 3 getDefaultSurface -> IOSurfaceID | confirmed |
| S3-sel-4 | 4 swapBegin -> swap token | confirmed |
| S3-sel-5 | 5 swapEnd <- `IOMobileFramebufferSwapArg` struct | confirmed |
| S3-sel-6 | 6 swapWait (token, waitOptions, timeout_millis since 4.2) | confirmed |
| S3-sel-7 | 7 getId | confirmed |
| S3-sel-8 | 8 getDisplaySize (`width`/`height` uint32, not `CGSize`) | confirmed |
| S3-sel-9 | 9 setVSyncNotifications (fn + userdata; 0 disables) | refuted |
| S3-sel-12 | 12 requestPowerChange | confirmed |
| S3-sel-15 | 15 setDebugFlags (`0xf`) | confirmed |
| S3-sel-17 | 17 setGammaTable (`0x11`, struct `0xc0c`) | confirmed |
| S3-sel-18 | 18 isMainDisplay | confirmed |
| S3-sel-19 | 19 setWhiteOnBlackMode (`0x13`) | confirmed |
| S3-sel-22 | 22 setDisplayDevice (`0x16`) | confirmed |
| S3-sel-27 | 27 getGammaTable (`0x1b`) | confirmed |
| S3-sel-33 | 33 setVideoPowerSaving (`0x21`; was 32 before 4.0) | confirmed |
| S3-sel-50 | 50 setBrightnessCorrection (`0x32`; was 49 before 8.0) | confirmed |
| S3-layers-4 | `NUM_LAYERS == 4` since iOS 7 | confirmed |
| S3-swaparg | `SwapArg` is 0x560 at `fb+0x18`. Token +0x98, surface IDs +0x9c, dest/src +0xac, layer mask +0x14c, bgColor +0x154, gamma +0x158 | confirmed |
| S3-vsync | Vsync is notify type 5 via EnableVSyncNotifications / sel 0x48, not wiki selector 9 | confirmed |

## S4. aiaf `_kern_*` map (macOS Sonoma 14.3)

Lead, not authority. Confirm each selector on **iOS 26.1**.

| Id | Claim | Status |
|---|---|---|
| S4-sel-3 | 3 GetLayerDefaultSurface (scalar) | confirmed |
| S4-sel-4 | 4 SwapBegin (scalar out token) | confirmed |
| S4-sel-5 | 5 SwapEnd struct. aiaf size `0x4fc` on Sonoma. Conflicts with vphone 0x560/0x588/0x6e0 | refuted |
| S4-sel-6 | 6 SwapWait and SwapWaitWithTimeout (3 scalars) | confirmed |
| S4-sel-7 | 7 GetID | confirmed |
| S4-sel-8 | 8 GetDisplaySize | confirmed |
| S4-sel-0xc | 0xc RequestPowerChange | confirmed |
| S4-sel-0x12 | 0x12 IsMainDisplay | confirmed |
| S4-sel-0x13 | 0x13 SetWhiteOnBlackMode | confirmed |
| S4-sel-0x11 | 0x11 SetGammaTable struct size `0xc0c` | confirmed |
| S4-sel-0x1b | 0x1b GetGammaTable | confirmed |
| S4-sel-0x1d | 0x1d GetDisplayArea | confirmed |
| S4-sel-0x1c | 0x1c GetDotPitch | confirmed |
| S4-sel-0x21 | 0x21 EnableDisableVideoPowerSavings | confirmed |
| S4-sel-0x32 | 0x32 SetBrightnessCorrection | confirmed |
| S4-sel-0x33 | 0x33 SetColorRemapMode | confirmed |
| S4-sel-0x39 | 0x39 GetColorRemapMode | confirmed |
| S4-cancel-0x34 | 0x34 SwapCancel (1 scalar token). Matches Wawona "52" if decimal | confirmed |
| S4-sel-0x14 | 0x14 `_kern_SwapSignal` exists. Public `SwapSignal` is a stub (`0xe00002c2`/`0xe00002c7`) | confirmed |
| S4-census | Census all `_kern_*` on guest. Bind public wrappers if they still exist | confirmed |

## S5. vphone CFW (guest may already be patched)

| Id | Claim | Status |
|---|---|---|
| S5-trampoline | Public `IOMobileFramebufferSwap*` are thin trampolines: `cbz x0; ldr xN,[x0,#slot]; cbz xN; braaz xN` onto `_kern_*` or `_virt_*` | confirmed |
| S5-kern-end-5 | `_kern_SwapEnd` is userclient method 5 | confirmed |
| S5-struct-size | Struct size is a kernel property: 26.1 base `0x560`, 26.4 `0x588`, iOS 27 native `0x6e0`. Userland 18.6.2 sent `0x514`, 26.0 sent `0x548` | confirmed |
| S5-virt | `_virt_SwapEnd` does no IOConnect; in-process callback. 27 `_virt_*` T symbols on this guest | confirmed |
| S5-force-kern | Force-kern rewrites trampoline first insn to `b _kern_Swap*`. Guest 26.1 still `cbz` trampolines (not applied). iOS 27 VZ CFW only. Do not ship | confirmed |
| S5-guest-patch | Our vphone 26.1 guest may already have the SwapEnd size patch. Diff vs stock 23B85 | refuted |

## S6. Conflicts the first Ghidra pass must settle

| Id | Conflict | Status |
|---|---|---|
| S6-setlayer-arity | 3-arg (2015 gists / RecordMyScreen) vs 6-arg (screendump / Wawona) | confirmed |
| S6-layer-count | 3 (Wawona) vs 4 (wiki iOS 7+) | confirmed |
| S6-swapend-shape | Public `SwapEnd(fb)` vs `_kern_SwapEnd` struct (0x4fc / 0x560 / 0x588) | confirmed |
| S6-size-type | `GetDisplaySize` returns `CGSize` vs `{uint32 w,h}` | confirmed |
| S6-power-polarity | gist YES=enabled vs Wawona `0`=disable savings | confirmed |
| S6-cancel-sel | SwapCancel "52" vs aiaf `0x34` | confirmed |
| S6-exclusive | Whether a real exclusive / disable-other-clients export exists | confirmed |

## Verification bar

- **confirmed**: Ghidra decompile of the named guest symbol (arity, `CGRect`
  passing, IOConnect selector/size) matches. Swap / power / restore also need
  a tipa or lab call that returns 0.
- **refuted**: guest 26.1 disagrees. Keep the old claim here with the refute
  note. Do not implement the dead shape.
- **absent**: symbol not in the 26.1 export list. Document, do not bind.

`docs/ABI.md` is our 26.1 truth after Ghidra. GitHub issues mirror this file.
`scripts/sync-github.sh` updates S1-S6 family issues and M0-M9 trackers.

Leftover `unconfirmed` rows are not missing Ghidra work:

- `S1-iland-bind` is M9. This crate ships `wwn_iomfb_*`. Wawona L4
  still links frozen `iland-iomfb`.
- `S5-force-kern` is confirmed as CFW-only. Guest 26.1 trampolines are
  still `cbz`/`ldr`/`braaz`. The crate does not rewrite them.
