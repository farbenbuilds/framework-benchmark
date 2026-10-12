{
  description = "Template for dual-libc server packaging";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
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
        ...
      }: let
        mkServer = packagePkgs: name:
          packagePkgs.stdenv.mkDerivation {
            pname = name;
            version = "0.1.0";
            src = ./.;
            installPhase = ''
              mkdir -p $out/bin
              cp server $out/bin/server
            '';
          };
      in {
        packages = {
          server-glibc = mkServer pkgs "server-glibc";
          server-musl = mkServer pkgs.pkgsMusl "server-musl";
        };
      };
    };
}
