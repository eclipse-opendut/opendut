{
  description = "A Nix-flake-based development environment for Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { ... }@inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
        );
      in
      {
        devShells.default = craneLib.devShell {
          # additional packages for the dev shell
          packages = with pkgs; [
            bacon
            cargo-deny
            cargo-nextest
            clippy
          ];
        };
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
