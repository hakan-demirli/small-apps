{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "difffenced";
  propagatedBuildInputs = [
    (pkgs.python3.withPackages (pythonPackages: with pythonPackages; [ yt-dlp ]))
  ];
  dontUnpack = true;
  installPhase = ''
    install -Dm755 ${./difffenced.py} $out/bin/difffenced;
  '';
}
