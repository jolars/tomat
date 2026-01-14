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
    pkgs.mdbook
    pkgs.mdbook-mermaid
    pkgs.mdbook-admonish
    pkgs.mdbook-pagetoc
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
