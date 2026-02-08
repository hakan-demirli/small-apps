{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "nmcli-transfer";
  propagatedBuildInputs = [ ];
  dontUnpack = true;
  installPhase = ''
    install -Dm755 ${./nmcli-transfer.sh} $out/bin/nmcli-transfer;
  '';
}
