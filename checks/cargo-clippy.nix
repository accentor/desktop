{
  accentor-desktop,
  rustPackages
}:

accentor-desktop.overrideAttrs (old: {
  name = "accentor-desktop-clippy-check";

  nativeBuildInputs = (old.nativeBuildInputs or []) ++ [
    rustPackages.clippy
  ];

  buildPhase = ''
    cargo clippy -j $NIX_BUILD_CORES --workspace --all-targets -- -D warnings
    touch $out
  '';

  checkPhase = "true";
  installPhase = "true";
})
