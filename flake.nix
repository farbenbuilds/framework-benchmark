{
  description = "High-performance framework and server benchmark platform";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {
    self,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem = {
        pkgs,
        system,
        ...
      }: let
        inherit (pkgs) lib;
        isLinux = pkgs.stdenv.hostPlatform.isLinux;
        releaseVersion = "0.1.0";

        rustOverlay = import inputs.rust-overlay;
        pkgsWithRust = import inputs.nixpkgs {
          inherit system;
          overlays = [rustOverlay];
        };

        rustToolchain = pkgsWithRust.rust-bin.stable."1.85.0".default.override {
          extensions = ["rust-src" "rust-analyzer" "clippy" "rustfmt"];
        };

        pkgsMusl =
          if isLinux
          then pkgs.pkgsMusl
          else null;

        source = lib.cleanSourceWith {
          src = self;
          filter = path: type: let
            name = baseNameOf path;
          in
            !lib.elem name [
              ".git"
              "target"
              "result"
              ".cargo"
            ];
        };

        mkPackage = packagePkgs:
          packagePkgs.rustPlatform.buildRustPackage {
            pname = "framework-benchmark";
            version = releaseVersion;
            src = source;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            nativeBuildInputs = [rustToolchain];
          };

        mkDevShell = packagePkgs: isMuslEnv:
          packagePkgs.mkShell {
            packages = [
              rustToolchain
              pkgs.cargo-edit
              pkgs.cargo-watch
              pkgs.ripgrep
              pkgs.jq
            ];
            BENCHMARK_ENV =
              if isMuslEnv
              then "musl"
              else "default";
          };
      in {
        packages =
          {
            default = mkPackage pkgs;
          }
          // lib.optionalAttrs isLinux {
            server-glibc = mkPackage pkgs;
            server-musl = mkPackage pkgsMusl;
          };

        devShells =
          {
            default = mkDevShell pkgs false;
          }
          // lib.optionalAttrs isLinux {
            musl = mkDevShell pkgsMusl true;
          };
      };
    };
}
