{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "homepage";

  src = ./.;

  propagatedBuildInputs = [
    (pkgs.python3.withPackages (pythonPackages: with pythonPackages; [ ]))
  ];

  buildPhase = ''
    substituteInPlace homepage.py \
      --replace 'SCRIPT_DIR / "static"' "'$out/share/homepage/static'"
  '';

  installPhase = ''
    mkdir -p $out/bin $out/share/homepage

    cp -r static $out/share/homepage/static

    cp homepage.py $out/homepage.py

    ln -s $out/homepage.py $out/bin/homepage
    chmod +x $out/bin/homepage
  '';
}
