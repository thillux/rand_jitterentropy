{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux.pkgs;

      inherit (pkgs) lib stdenv;

      jitterentropy_patched = pkgs.jitterentropy.overrideAttrs (
        _: prevAttrs: {
          version = "3.7.0";
          src = pkgs.fetchFromGitHub {
            owner = "thillux";
            repo = "jitterentropy-library";
            rev = "ntg1-2025-10-01";
            hash = "sha256-mgjJf82H0q8XoHSv7l+1kOd6o3i6VWoZ7wOW8c+tSOM=";
          };

          outputs = [
            "bin"
            "out"
            "dev"
            "lib"
          ];

          # for secure memory
          propagatedBuildInputs = with pkgs; [
            openssl
          ];
          postPatch = ''
            sed -i '/add_subdirectory(tests\/gcd)/d' CMakeLists.txt
          '';
          # better find openssl
          nativeBuildInputs = prevAttrs.nativeBuildInputs ++ [ pkgs.pkg-config ];
          # enables secure memory mode
          cmakeFlags = [
            "-DEXTERNAL_CRYPTO=OPENSSL"
          ] ++ lib.optionals stdenv.hostPlatform.isStatic [
            "-DBUILD_SHARED_LIBS=OFF"
          ] ++ lib.optionals (!stdenv.hostPlatform.isStatic) [
            "-DBUILD_SHARED_LIBS=ON"
          ];
        }
      );

      buildInputs = with pkgs; [
        jitterentropy_patched
      ];
      
      nativeBuildInputs = with pkgs; [
        pkg-config
        rustPlatform.bindgenHook
        cargo
        rustc
      ];
    in
    {
      packages.x86_64-linux = {
        inherit jitterentropy_patched;
      };

      devShells.x86_64-linux.default = pkgs.mkShell {
        inherit buildInputs;
        inherit nativeBuildInputs;
      };
    };
}
