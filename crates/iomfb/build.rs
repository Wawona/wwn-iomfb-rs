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

    let sdk = env::var("SDKROOT").ok().or_else(|| {
        Command::new("xcrun")
            .args(["--sdk", apple_sdk(&target), "--show-sdk-path"])
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            })
    });

    let mut swiftc = Command::new("swiftc");
    swiftc.arg("-c").arg(&metal_swift).arg("-o").arg(&metal_o);
    swiftc.arg("-parse-as-library");
    swiftc.arg("-whole-module-optimization");
    if let Some(sdk) = &sdk {
        swiftc.arg("-sdk").arg(sdk);
    }
    swiftc.arg("-target").arg(&target);
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
