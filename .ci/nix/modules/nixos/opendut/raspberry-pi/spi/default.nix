{
  lib,
  config,
  ...
}:

let
  cfg = config.opendut.raspberry-pi.spi;
  board = config.opendut.raspberry-pi.board;

  dtFilter =
    {
      pi-4 = "*bcm2711-rpi-4-b.dtb";
    }
    .${board} or (throw "referenced unsupported board in SPI config: ${board}");

  dtboFile =
    {
      # https://github.com/raspberrypi/firmware/raw/016253866bbd6c77fcb2a744a97d81e79ad4884f/boot/overlays/spi0-0cs.dtbo
      pi-4 = ./spi0-0cs.dtbo;
    }
    .${board} or (throw "referenced unsupported board in SPI config: ${board}");
in
{
  options.opendut.raspberry-pi.spi = {
    enable = lib.mkEnableOption "Enable SPI for the Raspberry Pi";
  };

  config = lib.mkIf cfg.enable {
    hardware.raspberry-pi."4".apply-overlays-dtmerge.enable = board == "pi-4";
    hardware.deviceTree = {
      enable = true;
      filter = dtFilter;
      overlays = [
        {
          name = "spi";
          dtboFile = dtboFile;
        }
      ];
    };

    users.groups.spi = { };

    services.udev.extraRules = ''
      SUBSYSTEM=="spidev", KERNEL=="spidev0.0", GROUP="spi", MODE="0660"
    '';
  };
}
