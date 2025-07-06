{ pkgs, ... }:
pkgs.rustPlatform.buildRustPackage {
  pname = "markdown-countdown-lsp";
  version = "0.1.0";

  src = ./lsp;
  cargoHash = "sha256-fZygYZJvHUIuX14v2A4eGbDUd8a2pz/oBs+bNUc4aFQ=";

  nativeBuildInputs = [ pkgs.pkg-config ];

  # buildInputs = [ pkgs.openssl ];

  meta = {
    description = "LSP Server to show countdown until a date as inlay hints";
  };
}
