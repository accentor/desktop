{
  description = "Accentor Desktop";
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.zst";
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    {
      packages = builtins.mapAttrs
        (system: pkgs: {
          accentor-desktop = pkgs.callPackage ./default.nix { };
          default = inputs.self.packages.${system}.accentor-desktop;
        })
        inputs.nixpkgs.legacyPackages;
      devShells = builtins.mapAttrs
        (system: pkgs':
          let
            pkgs = pkgs'.extend inputs.devshell.overlays.default;
          in
          {
            accentor-desktop = pkgs.callPackage ./shell.nix { inherit inputs; };
            default = inputs.self.devShells.${system}.accentor-desktop;
          }
        )
        inputs.nixpkgs.legacyPackages;
      checks = builtins.mapAttrs
        (system: pkgs':
          let
            pkgs = pkgs'.extend (self: super: { accentor-desktop = inputs.self.packages.${system}.default; });
          in
        {
          cargo-fmt = pkgs.callPackage ./checks/cargo-fmt.nix {};
          cargo-clippy = pkgs.callPackage ./checks/cargo-clippy.nix {};
        })
        inputs.nixpkgs.legacyPackages;
    };
}
