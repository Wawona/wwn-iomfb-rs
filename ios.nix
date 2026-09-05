# Device-only iOS staticlib. Wawona L4 callPackages this with iosToolchain.
# No wwn-iland. No registry key. Cited: Wawona/docs/wwn-repo-dag.md.
{
  lib,
  pkgs,
  iosToolchain,
  simulator ? false,
  ...
}:

assert lib.assertMsg (!simulator) "iomfb-ios is TrollStore/Sileo device-only";

let
  cargoTarget = "aarch64-apple-ios";
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    targets = [ cargoTarget ];
  };
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = "wwn-iomfb";
  version = "0.1.0";
  src = ./.;
  __noChroot = true;

  cargoLock.lockFile = ./Cargo.lock;
  cargoBuildFlags = [
    "--package"
    "iomfb-c"
    "--features"
    "apple-iomfb"
    "--target"
    cargoTarget
  ];
  CARGO_BUILD_TARGET = cargoTarget;
  doCheck = false;

  nativeBuildInputs = [
    pkgs.clang
    rustToolchain
  ];

  preConfigure = ''
    ${iosToolchain.mkIOSBuildEnv { simulator = false; minVersion = "15.0"; }}
    export NIX_CFLAGS_COMPILE=""
    export NIX_CXXFLAGS_COMPILE=""
    export NIX_LDFLAGS=""
    export IPHONEOS_DEPLOYMENT_TARGET="${iosToolchain.deploymentTarget}"
    export RUSTFLAGS="-C linker=$XCODE_CLANG -C link-arg=-isysroot -C link-arg=$SDKROOT -C link-arg=$APPLE_DEPLOYMENT_FLAG $RUSTFLAGS"
    export CC_aarch64_apple_ios="$XCODE_CLANG"
    export CXX_aarch64_apple_ios="$XCODE_CLANGXX"
    export CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$XCODE_CLANG"
  '';

  installPhase = ''
    runHook preInstall
    mkdir -p "$out/lib" "$out/include"
    cp "target/${cargoTarget}/release/libiomfb_c.a" "$out/lib/libwwn_iomfb.a"
    cp include/wwn_iomfb.h "$out/include/"
    cp include/iomfb.h "$out/include/"
    runHook postInstall
  '';

  meta = {
    description = "Wawona L4 iOS Mode B IOMFB sink (reconstructed, MIT)";
    license = lib.licenses.mit;
    platforms = lib.platforms.darwin;
  };
}
