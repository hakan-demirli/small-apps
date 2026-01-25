{ pkgs, ... }:
pkgs.stdenv.mkDerivation {
  name = "youtube_sync";
  propagatedBuildInputs = [
    (pkgs.python3.withPackages (pythonPackages: with pythonPackages; [ yt-dlp ]))
    pkgs.ffmpeg
  ];
  dontUnpack = true;
  installPhase = ''
    install -Dm755 ${./youtube_sync.py} $out/bin/youtube_sync;
  '';
}
