{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      # Use the same nixpkgs
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      gitignore,
      ...
    }:
    let
      pkgsDyn = nixpkgs.legacyPackages.x86_64-linux.pkgs;
      pkgs = nixpkgs.legacyPackages.x86_64-linux.pkgsStatic;

      inherit (pkgs) lib stdenv;

      jitterentropy_patched = pkgs.jitterentropy.overrideAttrs (
        _: prevAttrs: {
          version = "3.7.0";
          src = pkgs.fetchFromGitHub {
            owner = "smuellerDD";
            repo = "jitterentropy-library";
            rev = "e7bf6282407d1ea52815cdd7746b4c086c0b19af";
            hash = "sha256-PC8CQBRjJKWWfSLuEWyl09yjxZ9XS2ZGI7OMSFPwZ48=";
          };

          patches = [ ];

          # for secure memory
          propagatedBuildInputs = with pkgs; [
            openssl
          ];

          # better find openssl
          nativeBuildInputs = prevAttrs.nativeBuildInputs ++ [ pkgs.pkg-config ];
          # enables secure memory mode
          cmakeFlags = [
            "-DINTERNAL_TIMER=OFF"
            "-DEXTERNAL_CRYPTO=OPENSSL"
          ]
          ++ lib.optionals stdenv.hostPlatform.isStatic [
            "-DBUILD_SHARED_LIBS=OFF"
          ]
          ++ lib.optionals (!stdenv.hostPlatform.isStatic) [
            "-DBUILD_SHARED_LIBS=ON"
          ];
        }
      );

      buildInputs = with pkgs; [
        jitterentropy_patched
        openssl
      ];

      nativeBuildInputs = with pkgs; [
        pkg-config
        pkgsDyn.rustPlatform.bindgenHook
      ];
    in
    {
      formatter.x86_64-linux = pkgsDyn.nixfmt-tree;

      packages.x86_64-linux = {
        inherit jitterentropy_patched;
        rngd = pkgs.callPackage ./build.nix {
          jitterentropy = jitterentropy_patched;
          inherit buildInputs nativeBuildInputs;
          inherit (gitignore.lib) gitignoreSource;
        };
      };

      devShells.x86_64-linux.default = pkgs.mkShell {
        JITTERENTROPY_LIB_DIR = "${pkgs.jitterentropy}/lib";

        buildInputs = buildInputs ++ [
          pkgs.pkg-config
        ];
        nativeBuildInputs = nativeBuildInputs ++ [
          pkgs.rustc
          pkgs.cargo
          pkgs.clippy
        ];
      };
    };
}
