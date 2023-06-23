{
  description        = "vapour - general purpose cli tool for Steam related tasks";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.nightly.latest.default;
          rustc = pkgs.rust-bin.nightly.latest.default;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            pkg-config
            openssl
            rust-bin.nightly.latest.default
          ];
        };

        packages.default = rustPlatform.buildRustPackage {
          name = "vapour";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl
            rustPlatform.cargoSetupHook
          ];
          buildInputs = with pkgs; [
            openssl
          ];

          RUST_BACKTRACE=1;
        };
      }
    );
}
