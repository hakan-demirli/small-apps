{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "gen_typing_test";
  propagatedBuildInputs = [
    pkgs.swww
    (pkgs.python3.withPackages (
      pythonPackages: with pythonPackages; [
        #
      ]
    ))
  ];
  dontUnpack = true;

  src = ../gen_typing_test;

  installPhase = ''
    mkdir -p $out/bin
    cp -r $src/* $out/
    rm $out/default.nix
    ln -s $out/gen_typing_test.py $out/bin/gen_typing_test
    chmod +x $out/bin/gen_typing_test
  '';
}
