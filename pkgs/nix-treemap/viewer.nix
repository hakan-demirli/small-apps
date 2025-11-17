{
  pkgs ? import <nixpkgs> { },
}:

pkgs.buildNpmPackage {
  pname = "nix-treemap-visualizer";
  version = "1.0.0";

  src = ./.;

  npmDepsHash = "sha256-HLgTfHOlHVcWhlak8qS6iaZluxdtdj1m8ePRnmIhGbg=";

  dontNpmBuild = true;

  buildPhase = ''
    runHook preBuild
    node ./build.js
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp index.html $out/index.html
    runHook postInstall
  '';

  dontNpmPack = true;
}
