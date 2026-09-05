//! TrollStore (limited) vs jailbreak (full reconstructed ABI).
//!
//! TrollStore tipas stay userspace: no `IOConnect`, no factory / HDCP /
//! `KernelTests`. A tipa on a jailbroken phone is still TrollStore unless
//! the caller asks for jailbreak.
//!
//! Jailbreak binaries (under `/var/jb`, or an explicit channel) get the
//! full 153-export Apple `dlopen` path. That is the unlimited RE backend.
//! This crate does not link ElleKit.

use std::sync::atomic::{AtomicI32, Ordering};

/// Install / privilege channel. Not Mode A.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// TrollStore `.tipa`. Limited present. No IOMFB userclient.
    TrollStore = 0,
    /// Sileo / jailbreak. Full reconstructed IOMFB ABI.
    Jailbreak = 1,
}

pub const CHANNEL_AUTO: i32 = 2;

static FORCED: AtomicI32 = AtomicI32::new(-1);

impl Channel {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::TrollStore),
            1 => Some(Self::Jailbreak),
            _ => None,
        }
    }

    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Pin the process channel. `None` returns to auto-detect.
pub fn set(channel: Option<Channel>) {
    FORCED.store(
        channel.map(Channel::as_i32).unwrap_or(-1),
        Ordering::SeqCst,
    );
}

pub fn forced() -> Option<Channel> {
    Channel::from_i32(FORCED.load(Ordering::SeqCst))
}

/// Resolve the active channel. Default is TrollStore (limited).
pub fn current() -> Channel {
    if let Some(c) = forced() {
        return c;
    }
    resolve(
        std::env::var("WWN_IOMFB_CHANNEL").ok().as_deref(),
        std::env::var("WWN_IOMFB_APPLE").ok().as_deref(),
        std::env::current_exe()
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
            .as_deref()
            .unwrap_or(""),
        std::env::var("JB_ROOT_PATH").ok().as_deref(),
    )
}

pub fn full_re() -> bool {
    current() == Channel::Jailbreak
}

/// Auto-detect without applying [`set`].
pub fn detect() -> Channel {
    resolve(
        std::env::var("WWN_IOMFB_CHANNEL").ok().as_deref(),
        std::env::var("WWN_IOMFB_APPLE").ok().as_deref(),
        std::env::current_exe()
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
            .as_deref()
            .unwrap_or(""),
        std::env::var("JB_ROOT_PATH").ok().as_deref(),
    )
}

/// Pure resolver. A TrollStore tipa under
/// `/var/containers/Bundle/Application` stays limited even when `/var/jb`
/// exists on the device.
pub fn resolve(
    channel_env: Option<&str>,
    apple_env: Option<&str>,
    exe: &str,
    jb_root: Option<&str>,
) -> Channel {
    if let Some(raw) = channel_env {
        let v = raw.trim();
        if v.eq_ignore_ascii_case("jailbreak")
            || v.eq_ignore_ascii_case("jb")
            || v == "1"
        {
            return Channel::Jailbreak;
        }
        if v.eq_ignore_ascii_case("trollstore")
            || v.eq_ignore_ascii_case("tipa")
            || v == "0"
        {
            return Channel::TrollStore;
        }
    }
    if matches!(apple_env, Some(v) if v == "1" || v.eq_ignore_ascii_case("true")) {
        return Channel::Jailbreak;
    }
    if exe_is_jailbreak(exe, jb_root) {
        return Channel::Jailbreak;
    }
    Channel::TrollStore
}

fn exe_is_jailbreak(exe: &str, jb_root: Option<&str>) -> bool {
    const PREFIXES: &[&str] = &[
        "/var/jb/",
        "/var/LIY/",
        "/var/ulb/",
        "/usr/libexec/ellekit/",
    ];
    if PREFIXES.iter().any(|p| exe.starts_with(p) || exe.contains(p)) {
        return true;
    }
    if let Some(root) = jb_root {
        let root = root.trim();
        if !root.is_empty() {
            let prefix = if root.ends_with('/') {
                root.to_string()
            } else {
                format!("{root}/")
            };
            if exe.starts_with(&prefix) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipa_stays_limited_on_a_jailbroken_phone() {
        let exe = "/var/containers/Bundle/Application/AAAA/WawonaIomfbBench.app/WawonaIomfbBench";
        assert_eq!(
            resolve(None, None, exe, Some("/var/jb")),
            Channel::TrollStore
        );
    }

    #[test]
    fn jb_cli_under_var_jb_is_full_re() {
        let exe = "/var/jb/usr/local/bin/iomfb-live";
        assert_eq!(resolve(None, None, exe, None), Channel::Jailbreak);
    }

    #[test]
    fn env_jailbreak_wins() {
        assert_eq!(
            resolve(Some("jailbreak"), None, "/Applications/Foo.app/Foo", None),
            Channel::Jailbreak
        );
    }

    #[test]
    fn env_trollstore_wins_over_exe() {
        assert_eq!(
            resolve(
                Some("trollstore"),
                Some("1"),
                "/var/jb/usr/local/bin/iomfb-live",
                None
            ),
            Channel::TrollStore
        );
    }

    #[test]
    fn apple_env_is_jailbreak_alias() {
        assert_eq!(
            resolve(None, Some("1"), "/tmp/x", None),
            Channel::Jailbreak
        );
    }

    #[test]
    fn default_is_trollstore() {
        assert_eq!(resolve(None, None, "/tmp/host-test", None), Channel::TrollStore);
    }
}
