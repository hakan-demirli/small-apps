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
      ...
    }:
    let
      lib = nixpkgs.lib;
      findPackageDirs = path: lib.filterAttrs (name: type: type == "directory") (builtins.readDir path);
      allPackageNames = lib.attrNames (findPackageDirs ./pkgs);

      perSystem =
        flake-utils.lib.eachSystem
          (lib.filter (s: !(lib.hasInfix "darwin" s)) flake-utils.lib.defaultSystems)
          (
            system:
            let
              pkgs = import nixpkgs { inherit system; };

              allMyPackages = lib.genAttrs allPackageNames (
                name: pkgs.callPackage (./pkgs + "/${name}/default.nix") { }
              );

              enabledPackages = lib.filterAttrs (name: pkg: !(pkg.meta.broken or false)) allMyPackages;
            in
            {
              packages = enabledPackages // {
                default = pkgs.buildEnv {
                  name = "small-apps-bundle-${system}";
                  paths = lib.attrValues enabledPackages;
                  meta.description = "Build environment containing all enabled small-apps";
                };
              };
            }
          );

    in
    perSystem
    // {
      overlays.default =
        final: prev:
        let
          system = prev.system;
          systemPackages = perSystem.packages.${system} or { };
          packagesToAdd = lib.filterAttrs (name: _: name != "default") systemPackages;
        in
        packagesToAdd;
    };
}
