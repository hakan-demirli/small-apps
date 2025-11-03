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

      buildMyPackages =
        pkgs:
        let
          findPackageDirs = path: lib.filterAttrs (name: type: type == "directory") (builtins.readDir path);
        in
        lib.mapAttrs' (
          pkgName: _:
          let
            packagePath = ./pkgs + "/${pkgName}";
          in
          lib.nameValuePair pkgName (pkgs.callPackage "${packagePath}/default.nix" { })
        ) (findPackageDirs ./pkgs);

      myPackagesOverlay = final: prev: buildMyPackages final;

      supportedSystems = lib.filter (
        system: !(lib.hasInfix "darwin" system)
      ) flake-utils.lib.defaultSystems;
    in
    {
      overlays.default = myPackagesOverlay;
    }

    // flake-utils.lib.eachSystem supportedSystems (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ myPackagesOverlay ];
        };

        myPackages = buildMyPackages pkgs;

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
          ytdlpp = pkgs.yt-dlp;
        };
      }
    );
}
