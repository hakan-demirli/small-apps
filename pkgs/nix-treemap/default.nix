{
  pkgs ? import <nixpkgs> { },
}:
let
  viewerApp = import ./viewer.nix { inherit pkgs; };
in
pkgs.stdenv.mkDerivation {
  pname = "nix-treemap";
  version = "1.0.1";

  src = ./.;

  nativeBuildInputs = [ pkgs.makeWrapper ];

  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin
    substitute ./nix-treemap $out/bin/nix-treemap \
      --replace '@depsScript@' '${./deps.sh}' \
      --replace '@viewerHtml@' '${viewerApp}/index.html' \
      --replace '@jq@' '${pkgs.jq}/bin/jq'

    chmod +x $out/bin/nix-treemap

    wrapProgram $out/bin/nix-treemap \
      --prefix PATH : ${
        pkgs.lib.makeBinPath [
          pkgs.coreutils
          pkgs.nix
          pkgs.xdg-utils
          pkgs.gawk
          pkgs.gnused
          pkgs.gnugrep
        ]
      }

    runHook postInstall
  '';

  dontBuild = true;
  dontCheck = true;
}
