{
  pkgs,
  lib,
  config,
  ...
}:

let
  cfg = config.opendut.edgar;
in
{
  options.opendut.edgar = {
    enable = lib.mkEnableOption "Enable openDuT EDGAR";
  };

  config = lib.mkIf cfg.enable {
    opendut.can.enable = lib.mkDefault true;

    systemd.services.opendut-edgar = {
      description = "openDuT EDGAR";
      wantedBy = [ "multi-user.target" ];
      wants = [ "network-online.target" ];
      after = [ "network-online.target" ];
      path = with pkgs; [
        opendut.cannelloni
        opendut.edgar
        can-utils
        iproute2
      ];
      unitConfig = {
        ConditionPathExists = "/etc/opendut/edgar.toml";
        StartLimitIntervalSec = 5;
        StartLimitBurst = 0;
      };
      serviceConfig = {
        ExecStart = "${pkgs.opendut.edgar}/bin/opendut-edgar service";
        Restart = "always";
        RestartSec = "30s";
        User = "root";
        Group = "root";
      };
    };

    # TODO: are these files obsolute when the opendut.can module is enabled?
    # Provide the exact files opendut-edgar setup checks in CreateCanKernelModuleLoadRule
    environment.etc = {
      "modules-load.d/opendut-edgar-vcan.conf".text = "vcan\n";
      "modules-load.d/opendut-edgar-can_gw.conf".text = "can_gw\n";
      "modprobe.d/opendut-edgar-can_gw.conf".text = "options can_gw max_hops=2\n";
    };

    environment.systemPackages = with pkgs; [
      opendut.edgar
      opendut.cannelloni
    ];
  };
}
