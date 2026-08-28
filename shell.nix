{ devshell
, inputs
, alsa-lib
, gnumake
, nixpkgs-fmt
, rust-analyzer
, openssl
}:

devshell.mkShell {
  name = "Accentor Desktop";
  imports = [ "${inputs.devshell}/extra/language/c.nix" "${inputs.devshell}/extra/language/rust.nix" ];
  commands = [
    {
      name = "accentor-cli";
      category = "[build outputs]";
      help = "Run the built cli";
      command = ''cargo run --bin accentor-cli -- "$@"'';
    }
    {
      name = "accentord";
      category = "[build outputs]";
      help = "Run the built daemon";
      command = ''cargo run --bin accentord -- "$@"'';
    }
  ];
  packages = [ gnumake nixpkgs-fmt rust-analyzer ];
  language.c = {
    includes = [ openssl alsa-lib ];
    libraries = [ openssl alsa-lib ];
  };
}
