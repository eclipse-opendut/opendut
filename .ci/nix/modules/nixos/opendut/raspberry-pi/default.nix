{
  pkgs,
  lib,
  config,
  ...
}:

let
  cfg = config.opendut.raspberry-pi;
in
{
  imports = [
    ./can
    ./spi
  ];

  options.opendut.raspberry-pi = {
    enable = lib.mkEnableOption "Enable Raspberry Pi configs for openDuT";
    board = lib.mkOption {
      type = lib.types.enum [
        "pi-3"
        "pi-4"
      ];
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.board != "pi-3";
        message = "the pi-3 is not yet implemented within our modules";
      }
    ];

    boot = {
      kernelPackages = pkgs.linuxPackages;
      kernelParams = [
        "fsck.repair=yes"
      ];
      loader = {
        grub.enable = false;
        generic-extlinux-compatible.enable = true;
      };
      consoleLogLevel = 2;
      supportedFilesystems.zfs = lib.mkForce false;
      kernel.sysctl."vm.swappiness" = 10;
    };

    hardware = {
      enableAllHardware = lib.mkForce false;
      raspberry-pi = {
        firmware = {
          enable = true;
          uboot.enable = true;
        };
        # Default 512 MiB CMA in vc4-kms-v3d fails to allocate inside the 0..768 MiB
        # range because U-Boot places the initrd/FDT right below 512 MiB, causing the
        # kernel to fall back to 5 MiB CMA and vc4 fbdev/HDMI initialization to fail
        # with -ENOMEM. 384 MiB fits cleanly below the U-Boot initrd region.
        # Additionally, the vendor bcm2711 DTB wires HDMI audio DMA to brcm,bcm2711-dma
        # (fe007b00.dma), which has no driver in mainline pkgs.linuxPackages, causing
        # vc4_hdmi to defer probing forever (-517). Setting noaudio=true removes the
        # HDMI dmas property so vc4-drm binds and brings up HDMI video output cleanly.
        configtxt.deviceTreeOverlays.all = [
          {
            vc4-kms-v3d = {
              cma-384 = true;
              noaudio = true;
            };
          }
        ];
      };
    };
  };
}
