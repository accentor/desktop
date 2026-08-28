{
  accentor-desktop,
  rustPackages
}:

accentor-desktop.overrideAttrs (old: {
  name = "accentor-desktop-fmt-check";

  nativeBuildInputs = (old.nativeBuildInputs or []) ++ [
    rustPackages.rustfmt
  ];

  buildPhase = ''
    cargo fmt --all -- --check
    touch $out
  '';

  checkPhase = "true";
  installPhase = "true";
})
