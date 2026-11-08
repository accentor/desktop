{
  description = "Accentor Desktop Development";

  inputs = {
    super.url = "../";
    nixpkgs.follows = "super/nixpkgs";
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: {
    devShells = builtins.mapAttrs (
      system: pkgs':
      let
        pkgs = pkgs'.extend inputs.devshell.overlays.default;
      in
      {
        accentor-desktop = pkgs.callPackage ./shell.nix { inherit inputs; };
        default = inputs.self.devShells.${system}.accentor-desktop;
      }
    ) inputs.nixpkgs.legacyPackages;
  };
}
