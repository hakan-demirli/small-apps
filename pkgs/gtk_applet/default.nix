{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "gtk_applet";

  nativeBuildInputs = with pkgs; [
    wrapGAppsHook
    gobject-introspection
    libappindicator
  ];
  propagatedBuildInputs = [
    pkgs.activate-linux # for update_wp
    (pkgs.python3.withPackages (
      pythonPackages: with pythonPackages; [
        pygobject3
        requests
      ]
    ))
  ];
  dontUnpack = true;

  src = ../gtk_applet;

  installPhase = ''
    mkdir -p $out/bin
    cp -r $src/gtk_applet_power_menu.py $out/
    cp -r $src/gtk_applet_script_menu.py $out/
    ln -s $out/gtk_applet_power_menu.py $out/bin/gtk_applet_power_menu
    ln -s $out/gtk_applet_script_menu.py $out/bin/gtk_applet_script_menu
    chmod +x $out/bin/gtk_applet_script_menu
    chmod +x $out/bin/gtk_applet_power_menu
  '';
}
