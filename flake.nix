{
  description = "A collection of small applications";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = nixpkgs.lib;

        findPackageDirs = path: lib.filterAttrs (name: type: type == "directory") (builtins.readDir path);

        buildPackages =
          pkgsDir:
          lib.mapAttrs' (
            pkgName: _:
            let
              packagePath = "${toString pkgsDir}/${pkgName}";
            in
            lib.nameValuePair pkgName (pkgs.callPackage "${packagePath}/default.nix" { })
          ) (findPackageDirs pkgsDir);
        myPackages = buildPackages ./pkgs;
      in
      {
        packages = myPackages // {
          default = pkgs.buildEnv {
            name = "small-apps-bundle-${system}";
            paths = lib.attrValues myPackages;
            meta = {
              description = "Build environment containing all small-apps";
            };
          };
        };
        overlays.default = final: prev: myPackages;
      }
    );
}
