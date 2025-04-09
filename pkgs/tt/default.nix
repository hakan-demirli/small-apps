{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "tt";

  propagatedBuildInputs = [ (pkgs.python3.withPackages (pythonPackages: with pythonPackages; [ ])) ];
  dontUnpack = true;

  src = ../tt;

  installPhase = ''
    mkdir -p $out/bin
    cp -r $src/* $out/
    rm $out/default.nix
    ln -s $out/tt.py $out/bin/tt
    chmod +x $out/bin/tt
  '';
}
