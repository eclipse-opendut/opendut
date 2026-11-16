{
  pkgs,
  lib,
  config,
  ...
}:

let
  cfg = config.opendut.raspberry-pi.can.waveshare;
in
{
  options.opendut.raspberry-pi.can.waveshare = {
    enable = lib.mkEnableOption "Enable the Waveshare CAN hat for the Raspberry Pi";
  };

  config = lib.mkIf cfg.enable {
    opendut = {
      can.enable = lib.mkDefault true;
    };

    # Configure overlays via config.txt so the Raspberry Pi firmware applies them
    # https://github.com/NixOS/nixos-hardware/blob/919c16cf46a7acf9b44b2c2444bc39aa094eaa85/raspberry-pi/common/config-txt.nix
    # alongside vc4-kms-v3d before handing the device tree to U-Boot.
    hardware.raspberry-pi.configtxt.deviceTreeOverlays.pi4 = [
      # Equivalent to: dtoverlay=spi1-3cs,cs0_spidev=off
      {
        spi1-3cs = {
          cs0_spidev = false;
        };
      }

      # Equivalent to: dtoverlay=mcp251xfd,spi0-0,oscillator=40000000,interrupt=25
      {
        mcp251xfd = {
          spi0-0 = true;
          oscillator = 40000000;
          interrupt = 25;
        };
      }

      # Equivalent to: dtoverlay=mcp251xfd,spi1-0,oscillator=40000000,interrupt=24
      {
        mcp251xfd = {
          spi1-0 = true;
          oscillator = 40000000;
          interrupt = 24;
        };
      }
    ];

    # Enable systemd-networkd to manage network and CAN interfaces
    networking.useNetworkd = true;
    systemd.network.enable = true;

    # Automatically configure and bring up can0 and can1 in Classic CAN 2.0 mode
    systemd.network.networks = {
      "80-can0" = {
        matchConfig.Name = "can0";
        canConfig = {
          BitRate = "500K";
          FDMode = false;
          RestartSec = "100ms";
        };
        linkConfig.RequiredForOnline = false;
      };

      "80-can1" = {
        matchConfig.Name = "can1";
        canConfig = {
          BitRate = "500K";
          FDMode = false;
          RestartSec = "100ms";
        };
        linkConfig.RequiredForOnline = false;
      };
    };

    users.groups.spi = { };
    services.udev.extraRules = ''
      SUBSYSTEM=="spidev", KERNEL=="spidev0.0", GROUP="spi", MODE="0660"
    '';

    environment.systemPackages = with pkgs; [
      can-utils
    ];
  };
}
