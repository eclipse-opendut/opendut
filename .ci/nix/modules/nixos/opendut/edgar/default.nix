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
    useReleaseFallback = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Whether to use the pinned GitHub Release tarball instead of the locally built
        EDGAR distribution in `target/ci/distribution/`.
      '';
    };
    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.opendut.edgar.override {
        useReleaseFallback = cfg.useReleaseFallback;
      };
      defaultText = lib.literalExpression "pkgs.opendut.edgar";
      description = "The opendut-edgar package to use.";
    };
  };

  config = lib.mkIf cfg.enable {
    opendut.can.enable = lib.mkDefault true;

    systemd.services.opendut-edgar = {
      description = "openDuT EDGAR";
      wantedBy = [ "multi-user.target" ];
      wants = [ "network-online.target" ];
      after = [ "network-online.target" ];
      path = [
        pkgs.opendut.cannelloni
        cfg.package
        pkgs.can-utils
        pkgs.iproute2
      ];
      unitConfig = {
        ConditionPathExists = "/etc/opendut/edgar.toml";
        StartLimitIntervalSec = 5;
        StartLimitBurst = 0;
      };
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/opendut-edgar service";
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

    environment.systemPackages = [
      cfg.package
      pkgs.opendut.cannelloni
    ];
  };
}
