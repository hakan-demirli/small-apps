_: {
  nixpkgs.overlays = [
    (final: prev: {
      yt-dlp = prev.yt-dlp.overrideAttrs (oldAttrs: {
        version = "unstable-pr-13515";
        src = prev.fetchFromGitHub {
          owner = "coletdjnz";
          repo = "yt-dlp-dev";
          rev = "feat/youtube/sabr";
          hash = "sha256-uWyhJwzPNn9DccJJhEWF8eN3zcsnC91a+SjQFQ+Nba8=";
        };
      });

    })
  ];
}
