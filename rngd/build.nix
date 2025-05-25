{ pkgs
, buildInputs
, nativeBuildInputs
, rustPlatform
, gitignoreSource
}:

rustPlatform.buildRustPackage rec {
  pname = "jitter-rngd";
  version = "0.1.0";

  src = gitignoreSource ./.;

  doCheck = true;

  inherit buildInputs;
  inherit nativeBuildInputs;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
