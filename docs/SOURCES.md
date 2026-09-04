# Sources (cite, do not copy)

Bibliography for reconstructed IOMFB. Cite the URL or path. Do **not**
paste Apple headers, gist bodies, wiki C, or Ghidra decompiler output into
this repo.

Authority binary: guest `IOMobileFramebuffer` from vphone `wawona-jb`
(iOS 26.1 / 23B85). Host macOS DSC, Simulator, and the public iPhoneOS SDK
stub are not authority.

## Public reverse engineering

| Id | Source | Use |
|---|---|---|
| wiki | [Apple Wiki Dev:IOMobileFramebuffer](https://theapplewiki.com/wiki/Dev:IOMobileFramebuffer) | Userclient selectors + `SwapArg` layout, versioned |
| rms | [coolstar/RecordMyScreen `IOMobileFrameBuffer.h`](https://github.com/coolstar/RecordMyScreen/blob/master/RecordMyScreen/headers/IOMobileFrameBuffer.h) | 2008-era 3-arg `SwapSetLayer`, `CoreSurfaceBufferRef` (Zodttd / Steven Troughton-Smith) |
| gist-2015 | [anthonya1999 gist](https://gist.github.com/anthonya1999/e0bffcac15b6b208126d) | 2015 reconstructed userspace header; dlopen/dlsym |
| gist-nil | [NilStack fork](https://gist.github.com/NilStack/97dbc2bc05b8fa7c6095) | Same header, forked |
| gist-nevyn | [nevyn 64-bit gist](https://gist.github.com/nevyn/9486278) | `GetLayerDefaultSurface` on iOS 7 Retina |
| screendump | [cosmosgenius/screendump `Tweak.xm`](https://github.com/cosmosgenius/screendump/blob/master/Tweak.xm) | Modern 6-arg `SwapSetLayer` |
| aiaf | [aiaf `_kern_*` / Sonoma 14.3 IOConnect map](https://gist.github.com/aiaf/40cd11a9d2b46f92b655ce9819f83ad5) | Lead only. macOS, not iOS |
| fbvnc | FBVNCPublic entitlement + `GetMain` / `Swap*` draw path | Concepts. Wawona tipa rule cites them, not the sources |

## Wawona (frozen sink)

| Id | Path | Use |
|---|---|---|
| wwn-h | `wwn-iland/crates/wwn-iland-iomfb/include/IOMobileFramebuffer.h` | Reconstructed names Wawona calls today |
| wwn-m | `wwn-iland/crates/wwn-iland-iomfb/ffi/iomfb_platform.m` | dlopen trampoline + swap/restore |
| wwn-rs | `wwn-iland/crates/wwn-iland-iomfb/src/lib.rs` | Desktop sink (open / acquire / present / restore / hold) |
| wwn-present | `Wawona/src/platform/ios/WWNIlandPresenter.m` | Mode B present bind |
| wwn-docs | `Wawona/docs/iland-mode-a-b-desktop.md` | Product prose |
| wwn-hid | `Wawona/src/platform/ios/WWNModeBDisplayClaim.m` | HID park. **Not** IOMFB. Out of this crate |

## vphone CFW (do not ship)

| Id | Path | Use |
|---|---|---|
| cfw-swapend | `~/.vphone/src/vphone-cli/scripts/patchers/cfw_patch_iomfb_swapend.py` | Guest DSC `SwapEnd` size patch |
| cfw-force | `~/.vphone/src/vphone-cli/scripts/patchers/cfw_patch_iomfb_force_kern.py` | Force `_kern_Swap*` trampoline |

Guest 26.1 may already carry these patches. Diff guest IOMFB against the
stock 23B85 IPSW extract before treating guest bytes as stock Apple.

## Lab

| Id | Note |
|---|---|
| guest-dsc | vphone `wawona-jb` `/System/Library/Caches/com.apple.dyld/` |
| ipsw-23b85 | Same-build cross-check: `iPhone17,3_26.1_23B85`. Optional |
| kernel-uc | `IOMobileFramebufferUserClient`. `docs/kernel/` evidence only. Not linked |
