{ pkgs }:

pkgs.writeShellApplication {
  name = "flake-updater";

  runtimeInputs = [
    pkgs.git
    pkgs.gawk
    pkgs.gnused
    pkgs.nix
  ];

  text = builtins.readFile ./flake-updater.sh;
}
