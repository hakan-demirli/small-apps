{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "homepage";

  propagatedBuildInputs = [
    (pkgs.python3.withPackages (pythonPackages: with pythonPackages; [ ]))
  ];

  dontUnpack = true;

  src = ../homepage;

  installPhase = ''
    mkdir -p $out/bin
    cp -r $src/static          $out/bin/static
    cp -r $src/homepage.py     $out/

    ln -s $out/homepage.py     $out/bin/homepage
    chmod +x $out/bin/homepage
  '';
}
