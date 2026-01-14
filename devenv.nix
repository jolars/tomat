{
  pkgs,
  ...
}:

{
  packages = [
    pkgs.bashInteractive
    pkgs.go-task
    pkgs.llvmPackages.bintools
    pkgs.alsa-lib
    pkgs.cargo-llvm-cov
    pkgs.ffmpeg
    # pkgs.mdbook installed via cargo for latest version
    pkgs.mdbook-mermaid
    pkgs.mdbook-linkcheck
  ];

  languages.rust = {
    enable = true;
  };

  git-hooks = {
    hooks = {
      clippy = {
        enable = true;
        settings = {
          allFeatures = true;
        };
      };

      rustfmt = {
        enable = true;
      };
    };
  };
}
