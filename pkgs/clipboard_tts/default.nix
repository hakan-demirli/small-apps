{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "clipboard_tts";

  propagatedBuildInputs = [
    pkgs.piper-tts
    (pkgs.python3.withPackages (
      pythonPackages: with pythonPackages; [
        pyclip
      ]
    ))
  ];

  dontUnpack = true;

  installPhase = ''
    install -Dm755 ${./clipboard_tts.py} $out/bin/clipboard_tts;
  '';
}
