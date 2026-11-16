{
  pkgs,
}:

let
  version = "0.10.2";
  archMap = {
    "aarch64-linux" = {
      arch = "aarch64-unknown-linux-gnu";
      hash = "sha256-6DWzD1MRPbfeRS2hFKVdoFqkOaAsFH/jxI7GDXrUtL4=";
    };
    "x86_64-linux" = {
      arch = "x86_64-unknown-linux-gnu";
      hash = "sha256-gXCJPbTJlR3wsMIChtPV31abE9zZJbH3R1A61y/qa/Q=";
    };
  };
  target =
    archMap.${pkgs.stdenv.hostPlatform.system}
      or (throw "Unsupported system for opendut-edgar: ${pkgs.stdenv.hostPlatform.system}");
in
pkgs.stdenv.mkDerivation {
  pname = "opendut-edgar";
  inherit version;

  src = pkgs.fetchurl {
    url = "https://github.com/eclipse-opendut/opendut/releases/download/v${version}/opendut-edgar-${target.arch}-${version}.tar.gz";
    inherit (target) hash;
  };

  nativeBuildInputs = [ pkgs.autoPatchelfHook ];
  buildInputs = [ pkgs.stdenv.cc.cc.lib ];

  installPhase = ''
    runHook preInstall
    mkdir -p "$out/opt/opendut-edgar" "$out/bin"
    cp -r ./* "$out/opt/opendut-edgar/"
    chmod +x "$out/opt/opendut-edgar/opendut-edgar" "$out/opt/opendut-edgar/install/rperf"
    ln -s "$out/opt/opendut-edgar/opendut-edgar" "$out/bin/opendut-edgar"
    ln -s "$out/opt/opendut-edgar/install/rperf" "$out/bin/rperf"
    runHook postInstall
  '';
}
