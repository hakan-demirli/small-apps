{ pkgs, ... }:
let
  pythonEnv = pkgs.python3.withPackages (
    pythonPackages: with pythonPackages; [
      colorama
      pytest
    ]
  );
in
pkgs.stdenv.mkDerivation {
  name = "riveroftime";

  nativeBuildInputs = [
    pkgs.makeWrapper
    pythonEnv
  ];

  propagatedBuildInputs = [
    pythonEnv
  ];

  src = ./.;
  # we copy src/src/* so we don't need to unpack in the traditional sense if using a clean src, 
  # but standard behavior with src=path behaves well. 
  # user example used dontUnpack = true with src=./src. 
  # We will just let standard phases work or copy manually.
  # If we use dontUnpack=true, we can access $src directly.
  dontUnpack = true;

  doCheck = true;

  checkPhase = ''
    export PYTHONPATH=$src/src
    pytest $src/tests
  '';

  installPhase = ''
    mkdir -p $out/lib/riveroftime
    cp -r $src/src/* $out/lib/riveroftime/

    mkdir -p $out/bin

    makeWrapper ${pythonEnv}/bin/python $out/bin/riveroftime \
      --add-flags "-m riveroftime.cli" \
      --prefix PYTHONPATH : "$out/lib/riveroftime"
  '';
}
