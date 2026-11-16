{
  pkgs,
  lib,
  config,
  ...
}:

let
  cfg = config.opendut.can;
in
{
  options.opendut.can = {
    enable = lib.mkEnableOption "Enable CAN for openDuT";
  };

  config = lib.mkIf cfg.enable {
    boot = {
      kernelModules = [
        "can"
        "vcan"
        "can_gw"
      ];
      extraModprobeConfig = ''
        options can_gw max_hops=2
      '';
    };

    environment.systemPackages = with pkgs; [
      can-utils
    ];
  };
}
