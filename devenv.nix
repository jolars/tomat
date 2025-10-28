{
  pkgs,
  ...
}:

{
  packages = [
    pkgs.go-task
    pkgs.llvmPackages.bintools
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
