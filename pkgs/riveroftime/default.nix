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
