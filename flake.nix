{
  description = "A Pomodoro timer with daemon support for waybar and other status bars";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        tomat = pkgs.rustPlatform.buildRustPackage {
          pname = "tomat";
          version = "2.10.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            installShellFiles
            mdbook
          ];

          # Audio support requires ALSA on Linux
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.alsa-lib
          ];

          nativeCheckInputs = [
            pkgs.writableTmpDirAsHomeHook
          ];

          # Skip tests that require access to file system locations not available during Nix builds
          checkFlags = [
            # Skip tests that require access to file system locations not available during Nix builds
            "--skip=timer::tests::test_icon_path_creation"
            "--skip=timer::tests::test_notification_icon_config"
            "--skip=integration::"
          ];

          postInstall = ''
            installShellCompletion --cmd tomat \
              --bash target/completions/tomat.bash \
              --fish target/completions/tomat.fish \
              --zsh target/completions/_tomat

            installManPage target/man/*
          '';

          meta = with pkgs.lib; {
            description = "A Pomodoro timer with daemon support for waybar and other status bars";
            homepage = "https://jolars.github.io/tomat";
            license = licenses.mit;
            maintainers = [ ];
            mainProgram = "tomat";
          };
        };
      in
      {
        packages = {
          default = tomat;
          tomat = tomat;
        };

        apps = {
          default = {
            type = "app";
            program = "${tomat}/bin/tomat";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            go-task
            mdbook
            alsa-lib
          ];
        };
      }
    );
}
