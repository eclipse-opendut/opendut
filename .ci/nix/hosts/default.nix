{ inputs }:

let
  lib = inputs.nixpkgs.lib;

  mkRaspberryPiHost =
    vars:
    inputs.nixpkgs.lib.nixosSystem {
      specialArgs = {
        inherit inputs vars;
      };
      modules = [
        inputs.self.nixosModules.opendut
        {
          imports = [
            ./opendut-pi-base
          ];

          opendut = {
            edgar.enable = true;
            raspberry-pi = {
              enable = true;
              board = vars.board;
              can.waveshare = {
                enable = true;
                model = vars.waveshareModel;
              };
            };
          };
        }
      ]
      ++ lib.optional (vars.board == "pi-3") inputs.nixos-hardware.nixosModules.raspberry-pi-3
      ++ lib.optional (vars.board == "pi-4") inputs.nixos-hardware.nixosModules.raspberry-pi-4;
    };
in
{
  opendut-pi-1 = mkRaspberryPiHost {
    hostname = "opendut-pi-1";
    board = "pi-4";
    waveshareModel = "mcp251xfd";
  };
  opendut-pi-2 = mkRaspberryPiHost {
    hostname = "opendut-pi-2";
    board = "pi-4";
    waveshareModel = "mcp2515";
  };
}
