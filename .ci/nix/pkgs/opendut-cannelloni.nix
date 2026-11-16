{
  pkgs,
}:

let
  version = "1.1.0";
  archMap = {
    "aarch64-linux" = {
      arch = "linux-arm64-lts";
      hash = "sha256-BvHP6kIF78Z8axVRCcxkfIXe6sqrAPqL6YE4sk9WkXg=";
    };
    "x86_64-linux" = {
      arch = "manylinux_2_28-x64";
      hash = "sha256-+Y5MrTptHfVUI0E+pSxnbS6yqz14PyX3IGVPV5fIPEQ=";
    };
  };
  target =
    archMap.${pkgs.stdenv.hostPlatform.system}
      or (throw "Unsupported system for cannelloni-opendut: ${pkgs.stdenv.hostPlatform.system}");
in
pkgs.stdenv.mkDerivation {
  pname = "opendut-cannelloni";
  inherit version;

  src = pkgs.fetchurl {
    url = "https://github.com/eclipse-opendut/cannelloni-build/releases/download/v${version}/cannelloni_${target.arch}_${version}.tar.gz";
    inherit (target) hash;
  };

  nativeBuildInputs = [ pkgs.autoPatchelfHook ];
  buildInputs = [ pkgs.stdenv.cc.cc.lib ];

  installPhase = ''
    runHook preInstall
    install -Dm755 cannelloni "$out/bin/cannelloni"
    install -Dm755 libcannelloni-common.so.0 "$out/lib/libcannelloni-common.so.0"
    cp -P libsctp.so* "$out/lib/"
    runHook postInstall
  '';
}
