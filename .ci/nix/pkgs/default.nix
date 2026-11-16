{
  pkgs,
  inputs,
}:

let
  mkAarch64SdCardImageFromNixosConfiguration =
    nixosConfiguration:
    (nixosConfiguration.extendModules {
      modules = [
        (
          { modulesPath, vars, ... }:
          {
            imports = [ (modulesPath + "/installer/sd-card/sd-image-aarch64.nix") ];
            image.baseName = "nixos-aarch64-${vars.hostname}-sdcard";
            sdImage.compressImage = false;
            sdImage.firmwareSize = 128; # staging all vendor overlays + u-boot.bin fills nearly all of the default 30 MiB FAT partition.
          }
        )
      ];
    }).config.system.build.sdImage;
in
{
  opendut-cannelloni = pkgs.callPackage ./opendut-cannelloni.nix { };
  opendut-edgar = pkgs.callPackage ./opendut-edgar.nix { };

  opendut-pi-1-sd-image = mkAarch64SdCardImageFromNixosConfiguration inputs.self.nixosConfigurations.opendut-pi-1;
  opendut-pi-2-sd-image = mkAarch64SdCardImageFromNixosConfiguration inputs.self.nixosConfigurations.opendut-pi-2;
}
