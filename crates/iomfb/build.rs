use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    if !target.contains("apple") {
        return;
    }
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let ffi = PathBuf::from(&manifest).join("../../ffi");
    let surface_c = ffi.join("iomfb_surface.c");
    let metal_swift = ffi.join("iomfb_metal.swift");
    println!("cargo:rerun-if-changed={}", surface_c.display());
    println!("cargo:rerun-if-changed={}", metal_swift.display());

    let out_dir = env::var("OUT_DIR").unwrap();
    let metal_o = PathBuf::from(&out_dir).join("iomfb_metal.o");

    cc::Build::new()
        .file(&surface_c)
        .compile("iomfb_surface");

    // Prefer xcrun for this Cargo TARGET. Ambient SDKROOT is often iPhoneOS
    // during Mode B tipa builds; host build.rs then still targets macOS and
    // swiftc dies with "sysroot for iPhoneOS but targeting MacOSX".
    let wanted_sdk = apple_sdk(&target);
    let sdk = Command::new("xcrun")
        .args(["--sdk", wanted_sdk, "--show-sdk-path"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            env::var("SDKROOT").ok().filter(|root| {
                let lower = root.to_lowercase();
                match wanted_sdk {
                    "iphoneos" => lower.contains("iphoneos"),
                    "appletvos" => lower.contains("appletvos"),
                    "watchos" => lower.contains("watchos"),
                    "xros" => lower.contains("xros"),
                    "macosx" => lower.contains("macosx") || lower.contains("macossdk"),
                    _ => true,
                }
            })
        });

    // Cargo TARGET is aarch64-apple-ios (no OS version). swiftc then assumes
    // a prehistoric deployment floor and rejects Metal / IOSurface APIs.
    // Prefer IPHONEOS_DEPLOYMENT_TARGET (ios.nix) and an arm64-apple-iosN.M
    // triple matching the tipa / Mode B scripts.
    let swift_target = apple_swift_target(&target);
    let mut swiftc = Command::new("swiftc");
    swiftc.arg("-c").arg(&metal_swift).arg("-o").arg(&metal_o);
    swiftc.arg("-parse-as-library");
    swiftc.arg("-whole-module-optimization");
    if let Some(sdk) = &sdk {
        swiftc.arg("-sdk").arg(sdk);
    }
    swiftc.arg("-target").arg(&swift_target);
    let status = swiftc.status().expect("swiftc iomfb_metal");
    if !status.success() {
        panic!("swiftc failed compiling iomfb_metal.swift");
    }

    println!("cargo:rustc-link-lib=static=iomfb_surface");
    println!("cargo:rustc-link-arg={}", metal_o.display());
    println!("cargo:rustc-link-lib=framework=IOSurface");
    println!("cargo:rustc-link-lib=framework=Metal");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    if target.contains("ios") || target.contains("tvos") || target.contains("xros") {
        println!("cargo:rustc-link-lib=framework=UIKit");
    }
}

fn apple_sdk(target: &str) -> &'static str {
    if target.contains("ios") {
        "iphoneos"
    } else if target.contains("tvos") {
        "appletvos"
    } else if target.contains("watchos") {
        "watchos"
    } else if target.contains("xros") {
        "xros"
    } else {
        "macosx"
    }
}

fn apple_swift_target(cargo_target: &str) -> String {
    let ios_min = env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "13.0".into());
    let tv_min = env::var("TVOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "17.0".into());
    let vision_min = env::var("XROS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "26.0".into());
    let mac_min = env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "14.0".into());
    if cargo_target == "aarch64-apple-ios" {
        format!("arm64-apple-ios{ios_min}")
    } else if cargo_target == "aarch64-apple-ios-sim" {
        format!("arm64-apple-ios{ios_min}-simulator")
    } else if cargo_target == "aarch64-apple-tvos" {
        format!("arm64-apple-tvos{tv_min}")
    } else if cargo_target == "aarch64-apple-tvos-sim" {
        format!("arm64-apple-tvos{tv_min}-simulator")
    } else if cargo_target.contains("xros") && cargo_target.contains("sim") {
        format!("arm64-apple-xros{vision_min}-simulator")
    } else if cargo_target.contains("xros") {
        format!("arm64-apple-xros{vision_min}")
    } else if cargo_target.contains("darwin") || cargo_target.contains("macos") {
        format!("arm64-apple-macosx{mac_min}")
    } else {
        cargo_target.to_string()
    }
}
