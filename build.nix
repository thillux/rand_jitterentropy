{
  pkgs,
  buildInputs,
  nativeBuildInputs,
  rustPlatform,
  gitignoreSource,
  jitterentropy,
}:

rustPlatform.buildRustPackage rec {
  pname = "jitter-rngd";
  version = "0.2.0";

  src = gitignoreSource ./.;

  doCheck = true;

  inherit buildInputs;
  inherit nativeBuildInputs;

  JITTERENTROPY_LIB_DIR = "${jitterentropy}/lib";
  CARGO_TERM_VERBOSE = "true";

  cargoBuildFlags = [
    "--package"
    "jitter-rngd"
  ];

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
