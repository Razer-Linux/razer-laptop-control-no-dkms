{
  description = "razer-laptop-control";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        name = "razer-laptop-control";
      in rec {
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = name;
          version = "0.0.1";

          nativeBuildInputs = with pkgs; [pkg-config];
          buildInputs = with pkgs; [dbus.dev hidapi];

          src = ./razer_control_gui;

          postConfigure = ''
            substituteInPlace src/device.rs --replace '/usr/share/razercontrol/laptops.json' '${./razer_control_gui/data/devices/laptops.json}'
          '';

          postInstall = ''
            mkdir -p $out/lib/udev/rules.d
            cp ${./razer_control_gui/data/udev/99-hidraw-permissions.rules} $out/lib/udev/rules.d/99-hidraw-permissions.rules
          '';

          cargoLock = {
            lockFile = ./razer_control_gui/Cargo.lock;
          };
        };
        defaultPackage = packages.default;

        nixosModules.default = {
          config,
          lib,
          pkgs,
          ...
        }:
          with lib; let
            cfg = config.services.razer-laptop-control;
          in {
            options.services.razer-laptop-control = {
              enable = mkEnableOption "Enables razer-laptop-control";
              package = mkOption {
                type = types.package;
                default = packages.default;
              };
            };

            config = mkIf cfg.enable {
              services.upower.enable = true;

              services.udev.packages = [packages.default];

              systemd.user.services."razerdaemon" = {
                description = "Razer laptop control daemon";
                serviceConfig = {
                  Type = "simple";
                  ExecStart = "${packages.default}/bin/daemon";
                };
                wantedBy = ["default.target"];
              };
            };
          };
      }
    );
}
