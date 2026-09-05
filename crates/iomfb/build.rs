fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("apple") {
        return;
    }
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let src = format!("{manifest}/../../ffi/iomfb_surface.m");
    println!("cargo:rerun-if-changed={src}");
    cc::Build::new()
        .file(&src)
        .flag("-fobjc-arc")
        .flag("-fobjc-arc-exceptions")
        .compile("iomfb_surface");
    println!("cargo:rustc-link-lib=framework=IOSurface");
    println!("cargo:rustc-link-lib=framework=Metal");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    if target.contains("ios") || target.contains("tvos") || target.contains("xros") {
        println!("cargo:rustc-link-lib=framework=UIKit");
    }
}
