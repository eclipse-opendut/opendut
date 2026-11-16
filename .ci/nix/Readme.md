# openDuT Nix Configuration

This directory contains the Nixpkgs overlays, packages, NixOS modules, and host configurations for building openDuT systems (such as Raspberry Pi SD card images with EDGAR and CAN support).

## Structure

- `pkgs/` — Package definitions (`opendut-edgar`, `opendut-cannelloni`, and Raspberry Pi SD card image targets).
- `overlays/` — `overlays.opendut` exposing `pkgs.opendut.edgar` and `pkgs.opendut.cannelloni`.
- `modules/nixos/opendut/` — `nixosModules.opendut` providing options for EDGAR, CAN, and Raspberry Pi hardware/hats.
- `hosts/` — Reference NixOS host configurations (`opendut-pi-1`, `opendut-pi-2`) based on `hosts/opendut-pi-base`.

---

## Building the `opendut-pi-1` SD Card Image

From the repository root, build the SD card image for `opendut-pi-1`:

```bash
nix build .#opendut-pi-1-sd-image
```

> **Note (Cross-Compilation on `x86_64-linux`):**  
> Because the Raspberry Pi target is `aarch64-linux`, building on an `x86_64-linux` host requires `aarch64-linux` emulation (e.g. via `boot.binfmt.emulatedSystems = [ "aarch64-linux" ];` on NixOS) or a remote `aarch64-linux` builder.

Once built, the uncompressed image is located inside `./result/sd-image/`:

```bash
ls -lh result/sd-image/
```

### Flashing to an SD Card

Replace `/dev/sdX` with your target SD card block device:

```bash
sudo dd if=$(echo result/sd-image/*.img) of=/dev/sdX bs=4M status=progress conv=fsync
```

---

## Minimal Example: Creating Your Own Raspberry Pi Image

You can consume this repository's flake outputs (`nixosModules.opendut` and `overlays.opendut`) in your own `flake.nix` to build a custom Raspberry Pi SD card image:

```nix
{
  description = "Custom openDuT Raspberry Pi Image";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixos-hardware = {
      url = "github:NixOS/nixos-hardware";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    opendut = {
      url = "github:eclipse-opendut/opendut";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-hardware, opendut, ... }:
    let
      system = "aarch64-linux";
    in
    {
      nixosConfigurations.my-opendut-pi = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          nixos-hardware.nixosModules.raspberry-pi-4
          opendut.nixosModules.opendut
          ({ modulesPath, ... }: {
            imports = [
              (modulesPath + "/installer/sd-card/sd-image-aarch64.nix")
            ];

            system.stateVersion = "26.11";
            networking.hostName = "my-opendut-pi";

            nixpkgs.overlays = [
              opendut.overlays.opendut
            ];

            opendut = {
              edgar.enable = true;
              raspberry-pi = {
                enable = true;
                board = "pi-4";
                can.waveshare.enable = true;
              };
            };

            # SD card image settings
            sdImage.compressImage = false;
            sdImage.firmwareSize = 128;

            # Minimal access setup
            services.openssh.enable = true;
            users.users.root.openssh.authorizedKeys.keys = [
              "ssh-ed25519 AAAA... your-key"
            ];
          })
        ];
      };

      packages.${system}.my-opendut-pi-sd-image =
        self.nixosConfigurations.my-opendut-pi.config.system.build.sdImage;
    };
}
```

Build the custom image with:

```bash
nix build .#packages.aarch64-linux.my-opendut-pi-sd-image
```

---

## Available Module Options (`nixosModules.opendut`)

| Option | Type | Description |
|---|---|---|
| `opendut.edgar.enable` | `bool` | Enables the `opendut-edgar` systemd service (starts when `/etc/opendut/edgar.toml` exists), installs `opendut-edgar` and `opendut-cannelloni`, and enables `opendut.can.enable` by default. |
| `opendut.can.enable` | `bool` | Loads `can`, `vcan`, and `can_gw` (`max_hops=2`) kernel modules and installs `can-utils`. |
| `opendut.raspberry-pi.enable` | `bool` | Configures Raspberry Pi bootloader, firmware, U-Boot, and VC4 CMA settings. |
| `opendut.raspberry-pi.board` | `"pi-4"` \| `"pi-3"` | Target Raspberry Pi board model (`pi-4` supported). |
| `opendut.raspberry-pi.can.waveshare.enable` | `bool` | Configures device tree overlays (`spi1-3cs`, dual `mcp251xfd`) and `systemd-networkd` interfaces (`can0`, `can1` at 500K bitrate) for the Waveshare 2-CH CAN HAT. |
| `opendut.raspberry-pi.spi.enable` | `bool` | Enables SPI device tree overlay and udev rules for `spidev0.0`. |
