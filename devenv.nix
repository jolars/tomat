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
    pkgs.cargo-audit
    pkgs.cargo-deny
    pkgs.ffmpeg
    pkgs.mdbook
    pkgs.mdbook-mermaid
    pkgs.mdbook-linkcheck2
  ];

  languages.rust = {
    enable = true;
  };

  languages.javascript = {
    enable = true;
    package = pkgs.nodejs_24;
    pnpm.enable = true;
  };

  # Use the project's runtime version with the Nix loader on Linux.
  env = pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
    MINIFLARE_WORKERD_PATH = pkgs.writeShellScript "tomat-workerd" ''
      runtime="$(${pkgs.nodejs_24}/bin/node -p 'require(require.resolve("workerd", { paths: [require.resolve("wrangler/package.json")] })).default')"
      exec ${pkgs.stdenv.cc.bintools.dynamicLinker} --library-path ${pkgs.glibc}/lib "$runtime" "$@"
    '';
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
