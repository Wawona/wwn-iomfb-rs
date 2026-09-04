{
  description = "wwn-iomfb-rs: reconstructed iOS IOMobileFramebuffer (MIT). L3-prime, nixpkgs-only. No Apple code.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ];
      forAll = nixpkgs.lib.genAttrs systems;
    in {
      packages = forAll (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "wwn-iomfb-rs";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = true;
          };
        });

      checks = forAll (system: {
        iomfb = self.packages.${system}.default;
      });
    };
}
