# TrollStore vs jailbreak

This crate has two channels. They are install privileges, not Mode A.

| Channel | Who | Backend | Full RE |
|---|---|---|---|
| **TrollStore** (default) | `.tipa` in `/var/containers/Bundle/Application` | Userspace IOSurface + present callback | No |
| **Jailbreak** | Sileo / `/var/jb` CLI / tweak | Apple `dlopen` IOMFB, all 153 exports | Yes |

TrollStore limitation is the product contract: no `IOConnect`, no
`IOMobileFramebufferUserClient`, no factory / HDCP / `KernelTests` /
`live_call` of Apple symbols. SpringBoard still composites the app
window. A tipa on a jailbroken phone stays TrollStore unless the
caller asks for jailbreak.

Jailbreak is unlimited reconstructed ABI. `GetMainDisplay`, swap,
power, vsync, factory, HDCP, and `iomfb_live_call` go to the guest
image. That is the full RE path. This crate does not link ElleKit.

## Select

1. `iomfb_channel_set(IOMFB_CHANNEL_TROLLSTORE | JAILBREAK | AUTO)`
2. `WWN_IOMFB_CHANNEL=trollstore|jailbreak`
3. `WWN_IOMFB_APPLE=1` (legacy alias for jailbreak)
4. Auto: executable under `/var/jb`, `/var/LIY`, or `JB_ROOT_PATH`

`Display::main()` / `iomfb_display_open_main` follow that order.
Explicit opens ignore detect:

```text
Display::trollstore(w, h) / iomfb_display_open_trollstore
Display::jailbreak_main() / iomfb_display_open_jailbreak
```

`iomfb_full_re()` is 1 only on the jailbreak channel.

## Build

TrollStore tipas: `cargo build -p iomfb-c --target aarch64-apple-ios`
(no `apple-iomfb` feature). Cannot `dlopen` Apple even if someone
sets the env.

Jailbreak CLIs: `--features apple-iomfb` plus
`iomfb_channel_set(IOMFB_CHANNEL_JAILBREAK)`.

## Guest proof (vphone `wawona-jb`, 2026-09-04)

Jailbreak (`iomfb-live --probe`, `apple-iomfb`, under `/var/jb`):

```text
channel=1 full_re=1 bound=153/153
open_jailbreak rc=0 userland=0 jailbreak=1
display 1290x2796
```

TrollStore (`iomfb-userland`, no `apple-iomfb`, forced channel 0):

```text
userland=1 bound=0 has_metal=1 sid=4 zero=1
PASS trollstore channel=0 full_re=0 no-IOConnect
```
