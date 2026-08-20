{ rustPlatform
, lib
, alsa-lib
, gnumake
, pkg-config
}:

let
  cargoToml = lib.importTOML ./Cargo.toml;
  version = cargoToml.workspace.package.version;
  rootFiles = lib.fileset.fileFilter (file: builtins.elem file.name [ "Cargo.lock" "Cargo.toml" ]) ./.;
  srcFiles = lib.fileset.unions [ rootFiles ./api ./cli ./communication ./server ./utils ];
  src = lib.fileset.toSource { root = ./.; fileset = srcFiles; };
in
rustPlatform.buildRustPackage {
  pname = "accentor-desktop";
  inherit version src;
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = [ pkg-config gnumake ];
  buildInputs = [ alsa-lib ];
}
