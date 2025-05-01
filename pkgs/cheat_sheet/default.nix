{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "cheat_sheet";
  version = "1.0";

  propagatedBuildInputs = [
    pkgs.bash
    pkgs.fzf
    pkgs.coreutils
  ];

  dontUnpack = true;
  dontFixup = true;

  src = ./cheat_sheet.sh;

  installPhase = ''
    install -Dm755 ${./cheat_sheet.sh} $out/bin/cheat_sheet;
  '';
}
