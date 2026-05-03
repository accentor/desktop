{
  description = "Accentor Desktop";
  inputs = {
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, devshell }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ devshell.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        cargoToml = pkgs.lib.importTOML ./Cargo.toml;
        version = cargoToml.workspace.package.version;
      in
        {
          packages = rec {
            default = accentor-desktop;
            accentor-desktop = pkgs.rustPlatform.buildRustPackage {
              pname = "accentor-desktop";
              inherit version;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
              nativeBuildInputs = [ pkgs.pkg-config pkgs.gnumake ];
              buildInputs = [ pkgs.alsa-lib pkgs.libsixel ];
            };
          };
          devShells = rec {
            default = accentor-desktop;
            accentor-desktop = pkgs.devshell.mkShell {
              name = "Accentor Desktop";
              imports = [ "${inputs.devshell}/extra/language/c.nix" "${inputs.devshell}/extra/language/rust.nix" ];
              packages = [ pkgs.gnumake ];
              language.c = {
                includes = with pkgs; [ openssl alsa-lib libsixel ];
                libraries = with pkgs; [ openssl alsa-lib libsixel ];
              };
            };
          };
        }
    );
}
