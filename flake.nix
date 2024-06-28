{
  description = "Simple text preprocessor for content segmentation";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
    in
    {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = manifest.name;
        version = manifest.version;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
        src = pkgs.lib.cleanSource ./.;
      };
      devShells.${system}.default = pkgs.mkShell {
        inherit nixpkgs;
        buildInputs = [ pkgs.rustc ];
      };
    };
}
