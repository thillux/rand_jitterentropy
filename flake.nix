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

      buildInputs = with pkgs; [
        jitterentropy
      ];

      nativeBuildInputs = with pkgs; [
        pkg-config
        pkgsDyn.rustPlatform.bindgenHook
      ];
    in
    {
      formatter.x86_64-linux = pkgsDyn.nixfmt-tree;

      packages.x86_64-linux = {
        default = self.packages.x86_64-linux.rngd;
        rngd = pkgs.callPackage ./build.nix {
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
