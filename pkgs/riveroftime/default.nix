{
  pkgs,
  lib,
}:
let
  realDerivation = pkgs.callPackage ./package.nix { };
  checks = import ./nix/checks.nix {
    inherit pkgs;
  };

in
pkgs.stdenv.mkDerivation {
  name = "riveroftime-checked";

  dontUnpack = true;
  dontBuild = true;

  doCheck = true;
  checkPhase = ''
    echo "Running checks..."
    ${builtins.concatStringsSep "\n" (builtins.map (c: "echo ${c}") (builtins.attrValues checks))}
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp ${realDerivation}/bin/riveroftime $out/bin/riveroftime
  '';

  meta = {
    description = "Wayland layer countdown timer";
    license = lib.licenses.mit;
    mainProgram = "riveroftime";
  };
}
