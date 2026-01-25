{
  pkgs,
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "riveroftime";
  version = "0.1.0";

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [
    pkgs.pkg-config
  ];

  buildInputs = [
    pkgs.wayland
    pkgs.libxkbcommon
    pkgs.fontconfig
  ];
}
