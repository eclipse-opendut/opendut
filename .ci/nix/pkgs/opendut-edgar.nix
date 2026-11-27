{
  pkgs,
  lib ? pkgs.lib,
  useReleaseFallback ? false,
}:

let
  cargoToml = builtins.fromTOML (builtins.readFile ../../../Cargo.toml);
  workspaceVersion = cargoToml.workspace.package.version;

  releaseFallback = {
    version = "0.10.2";
    hashes = {
      "aarch64-linux" = "sha256-6DWzD1MRPbfeRS2hFKVdoFqkOaAsFH/jxI7GDXrUtL4=";
      "x86_64-linux" = "sha256-gXCJPbTJlR3wsMIChtPV31abE9zZJbH3R1A61y/qa/Q=";
    };
  };

  archMap = {
    "aarch64-linux" = "aarch64-unknown-linux-gnu";
    "x86_64-linux" = "x86_64-unknown-linux-gnu";
  };

  system = pkgs.stdenv.hostPlatform.system;
  targetArch = archMap.${system} or (throw "Unsupported system for opendut-edgar: ${system}");

  tarballName = ver: "opendut-edgar-${targetArch}-${ver}.tar.gz";

  pwd = builtins.getEnv "PWD";
  envTarball = builtins.getEnv "OPENDUT_EDGAR_TARBALL";
  useReleaseEnv = (builtins.getEnv "OPENDUT_EDGAR_USE_RELEASE") != "";

  defaultLocalTarballPath =
    if envTarball != "" then
      envTarball
    else if pwd != "" then
      "${pwd}/target/ci/distribution/${targetArch}/${tarballName workspaceVersion}"
    else
      "";

  hasLocalTarball = defaultLocalTarballPath != "" && builtins.pathExists defaultLocalTarballPath;

  useFallback = useReleaseFallback || useReleaseEnv;

  version = if useFallback then releaseFallback.version else workspaceVersion;

  src =
    if !useFallback && hasLocalTarball then
      builtins.path {
        path = defaultLocalTarballPath;
        name = tarballName workspaceVersion;
      }
    else if useFallback then
      pkgs.fetchurl {
        url = "https://github.com/eclipse-opendut/opendut/releases/download/v${releaseFallback.version}/${tarballName releaseFallback.version}";
        hash = releaseFallback.hashes.${system};
      }
    else
      throw ''
        Local opendut-edgar distribution archive not found for ${targetArch} (version ${workspaceVersion}).

        Note: Because `target/` is git-ignored, Nix requires `--impure` to access local build artifacts:
          nix build --impure .#opendut-pi-1-sd-image

        Expected file at:
          target/ci/distribution/${targetArch}/${tarballName workspaceVersion}

        Either:
          1. Build the local distribution first:
               cargo ci edgar distribution --target ${targetArch}
             and run nix with `--impure`, or
          2. Specify a custom tarball path:
               OPENDUT_EDGAR_TARBALL=/path/to/${tarballName workspaceVersion} nix build --impure ...
          3. Use the GitHub release fallback (v${releaseFallback.version}):
               OPENDUT_EDGAR_USE_RELEASE=1 nix build --impure ...
             (or override `opendut.edgar.override { useReleaseFallback = true; }`)
      '';
in
pkgs.stdenv.mkDerivation {
  pname = "opendut-edgar";
  inherit version src;

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
