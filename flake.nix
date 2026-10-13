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
    flake-utils.lib.eachSystem
      [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ]
      (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          tomat = pkgs.rustPlatform.buildRustPackage {
            pname = "tomat";
            version = "2.14.0";

            src = pkgs.lib.cleanSourceWith {
              src = ./.;
              filter =
                path: type:
                let
                  baseName = baseNameOf path;
                in
                # Exclude docs directory to avoid mdbook build issues
                !(type == "directory" && baseName == "docs");
            };

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              installShellFiles
            ];

            # Audio support requires ALSA on Linux
            buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
              pkgs.alsa-lib
            ];

            nativeCheckInputs = [
              pkgs.writableTmpDirAsHomeHook
            ];

            checkFlags = [
              # Skip tests that require access to file system locations not available during Nix builds.
              "--skip=timer::tests::test_icon_path_creation"
              "--skip=timer::tests::test_notification_icon_config"
              "--skip=integration::"
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
              # Apple's plutil is unavailable in the Nix build sandbox.
              "--skip=service::tests::launchd_service_is_a_valid_plist"
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
              homepage = "https://tomat.cc";
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
            buildInputs =
              (with pkgs; [
                cargo
                rustc
                rustfmt
                clippy
                rust-analyzer
                go-task
                mdbook
              ])
              ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.alsa-lib ];
          };
        }
      )
    // {
      overlays.default = final: prev: {
        tomat = self.packages.${prev.system}.tomat;
      };
    };
}
