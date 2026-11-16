{
  description = "openDuT Nix flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixos-hardware = {
      url = "github:NixOS/nixos-hardware";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { ... }@inputs:
    {
      overlays = import ./.ci/nix/overlays { inherit inputs; };
      nixosModules.opendut = import ./.ci/nix/modules/nixos/opendut;
      nixosConfigurations = import ./.ci/nix/hosts { inherit inputs; };
    }
    // inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ (import inputs.rust-overlay) ];
        };
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
        );
      in
      {
        devShells.default = craneLib.devShell {
          packages = with pkgs; [
            bacon
            cargo-deny
            clippy
          ];
        };
        formatter = pkgs.nixfmt-rfc-style;
        packages = import ./.ci/nix/pkgs { inherit pkgs inputs; };
      }
    );
}
