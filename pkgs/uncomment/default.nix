{
  pkgs,
}:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "uncomment";
  version = "2.10.4";

  src = pkgs.fetchFromGitHub {
    owner = "hakan-demirli";
    repo = "uncomment";
    rev = "a9a1dc7c10cb983c62d4dcd1134b85fdb69a998e";
    hash = "sha256-a0sEVazu1JtF3UvtL4lDB70ltZkhC8fls/KdAGFX2wc=";
  };

  cargoHash = "sha256-ALRZK4o1jdWPfk412NxdyjElP9VBtgsyUnFq/UPGlF4=";
  doCheck = false;
}
