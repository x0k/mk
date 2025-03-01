{
  description = "Simple text preprocessor for content segmentation";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-24.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        f =
          with fenix.packages.${system};
          combine [
            stable.toolchain
            # targets.wasm32-unknown-unknown.stable.rust-std
          ];
      in
      {
        packages.default =
          (pkgs.makeRustPlatform {
            cargo = f;
            rustc = f;
          }).buildRustPackage
            {
              pname = manifest.name;
              version = manifest.version;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };
              src = pkgs.lib.cleanSource ./.;
            };
        devShells.default = pkgs.mkShell {
          inherit nixpkgs;
          buildInputs = [
            f
          ];
        };
      }
    );
}
