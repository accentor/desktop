{
  description = "Accentor Desktop";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = inputs: {
    packages = builtins.mapAttrs (system: pkgs: {
      accentor-desktop = pkgs.callPackage ./default.nix { };
      default = inputs.self.packages.${system}.accentor-desktop;
    }) inputs.nixpkgs.legacyPackages;
    overlays.default = (
      self: super: {
        accentor-desktop = self.callPackage ./default.nix { };
      }
    );
    checks = builtins.mapAttrs (
      system: pkgs':
      let
        pkgs = pkgs'.extend inputs.self.overlays.default;
      in
      {
        cargo-fmt = pkgs.callPackage ./checks/cargo-fmt.nix { };
        cargo-clippy = pkgs.callPackage ./checks/cargo-clippy.nix { };
      }
    ) inputs.nixpkgs.legacyPackages;
  };
}
