{ pkgs, ... }:

let
  htmlPreviewLsp = pkgs.rustPlatform.buildRustPackage {
    pname = "html-preview-lsp";
    version = "0.1.0";

    src = ./lsp;
    useFetchCargoVendor = true;
    cargoHash = "sha256-UTdOk/DMxTGhHbyTiIPz1V/wI6bTHDSGfO5n3b8NVBc=";

    propagatedBuildInputs = [
      pkgs.glibc
      pkgs.openssl
      pkgs.pkg-config
      pkgs.rustc
      pkgs.cargo
    ];

    nativeBuildInputs = [
      pkgs.glibc
      pkgs.openssl
      pkgs.pkg-config
      pkgs.rustc
      pkgs.cargo
    ];
  };

  htmlPreviewServer = pkgs.stdenv.mkDerivation {
    name = "html-preview-server";

    propagatedBuildInputs = [
      (pkgs.python3.withPackages (
        pythonPackages: with pythonPackages; [
          requests
          flask
          flask-cors
          flask-socketio
          eventlet
        ]
      ))
    ];

    dontUnpack = true;
    src = ./server;

    installPhase = ''
      mkdir -p $out/bin
      cp -r $src/main.py $out
      ln -s $out/main.py $out/bin/html-preview-server
      chmod +x $out/bin/html-preview-server
    '';
  };
in
{
  inherit htmlPreviewLsp htmlPreviewServer;
}
