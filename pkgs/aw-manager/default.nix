{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "aw-manager";

  propagatedBuildInputs = [
    (pkgs.python3.withPackages (
      pythonPackages: with pythonPackages; [
        requests
      ]
    ))
  ];

  dontUnpack = true;

  installPhase = ''
    install -Dm755 ${./aw-manager.py} $out/bin/aw-manager;
  '';
}
