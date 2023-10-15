{
  description = "razer-laptop-control";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        name = "razer-laptop-control";
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = name;
          version = "0.2.0";

          nativeBuildInputs = with pkgs; [pkg-config];
          buildInputs = with pkgs; [dbus.dev hidapi systemd];

          src = ./razer_control_gui;

          postConfigure = ''
            substituteInPlace src/device.rs --replace '/usr/share/razercontrol/laptops.json' '${./razer_control_gui/data/devices/laptops.json}'
          '';

          postInstall = ''
            mkdir -p $out/lib/udev/rules.d
            mkdir -p $out/libexec
            mv $out/bin/daemon $out/libexec
            cp ${./razer_control_gui/data/udev/99-hidraw-permissions.rules} $out/lib/udev/rules.d/99-hidraw-permissions.rules
          '';

          cargoLock = {
            lockFile = ./razer_control_gui/Cargo.lock;
          };
        };
      }
    )
    // {
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
              default = inputs.self.packages.${pkgs.stdenv.hostPlatform.system}.default;
            };
          };

          config = mkIf cfg.enable {
            services.upower.enable = true;
            environment.systemPackages = [cfg.package];
            services.udev.packages = [cfg.package];

            systemd.user.services."razerdaemon" = {
              description = "Razer laptop control daemon";
              serviceConfig = {
                Type = "simple";
                ExecStartPre = "${pkgs.coreutils}/bin/mkdir -p %h/.local/share/razercontrol";
                ExecStart = "${cfg.package}/libexec/daemon";
              };
              wantedBy = ["default.target"];
            };
          };
        };
    };
}
