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
    model = lib.mkOption {
      type = lib.types.enum [
        "mcp251xfd"
        "mcp2515"
      ];
      default = "mcp251xfd";
      description = ''
        Waveshare 2-CH CAN HAT controller model:
        - `mcp251xfd`: Waveshare 2-CH CAN FD HAT (MCP2518FD, 40 MHz oscillator, SPI0.0 + SPI1.0)
          https://www.waveshare.com/wiki/2-CH_CAN_FD_HAT
        - `mcp2515`: Waveshare 2-CH CAN HAT (MCP2515, 16 MHz oscillator, SPI0.0 + SPI0.1)
          https://www.waveshare.com/wiki/2-CH_CAN_HAT
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    opendut = {
      can.enable = lib.mkDefault true;
    };

    # Configure overlays via config.txt so the Raspberry Pi firmware applies them
    # https://github.com/NixOS/nixos-hardware/blob/919c16cf46a7acf9b44b2c2444bc39aa094eaa85/raspberry-pi/common/config-txt.nix
    # alongside vc4-kms-v3d before handing the device tree to U-Boot.
    hardware.raspberry-pi.configtxt.deviceTreeOverlays.pi4 =
      if cfg.model == "mcp251xfd" then
        [
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
        ]
      else if cfg.model == "mcp2515" then
        [
          # Equivalent to: dtoverlay=mcp2515-can0,oscillator=16000000,interrupt=23
          {
            mcp2515-can0 = {
              oscillator = 16000000;
              interrupt = 23;
            };
          }

          # Equivalent to: dtoverlay=mcp2515-can1,oscillator=16000000,interrupt=25
          {
            mcp2515-can1 = {
              oscillator = 16000000;
              interrupt = 25;
            };
          }
        ]
      else
        throw "Unsupported Waveshare CAN HAT model: ${cfg.model}";

    # Ensure spi0 (spi_bcm2835) probes before spi1 (spi_bcm2835aux) so that
    # can0 is deterministically assigned to spi0.0 and can1 to spi1.0 across reboots.
    boot.extraModprobeConfig = lib.mkIf (cfg.model == "mcp251xfd") ''
      softdep spi_bcm2835aux pre: spi_bcm2835
    '';

    # Enable systemd-networkd to manage network and CAN interfaces
    networking.useNetworkd = true;
    systemd.network.enable = true;

    systemd.network.links = lib.mkIf (cfg.model == "mcp251xfd") {
      "10-can0" = {
        matchConfig = {
          Path = "platform-fe204000.spi"; # spi0.0
          Type = "can";
        };
        extraConfig = ''
          AlternativeName = can-spi0
        '';
      };
      "10-can1" = {
        matchConfig = {
          Path = "platform-fe215080.spi"; # spi1.0
          Type = "can";
        };
        extraConfig = ''
          AlternativeName = can-spi1
        '';
      };
    };

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
