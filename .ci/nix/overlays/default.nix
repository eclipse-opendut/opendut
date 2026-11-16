{ inputs }:

let
  lib = inputs.nixpkgs.lib;

  # filter the attrset for keys that have the specified prefix, and remove this prefix from the key
  filterAttrsWithPrefix =
    prefix: attrs:
    lib.mapAttrs' (name: value: lib.nameValuePair (lib.removePrefix prefix name) value) (
      lib.filterAttrs (name: _: lib.hasPrefix prefix name) attrs
    );
in
{

  ########## openDuT packages #####################################################################

  opendut = final: prev: {
    opendut =
      prev.opendut or { }
      // filterAttrsWithPrefix "opendut-" (
        import ../pkgs {
          inherit inputs;
          pkgs = final;
        }
      );
  };
}
