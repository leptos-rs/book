{
  description = "Leptos Book";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.zst";
    nixpkgs_25_11.url = "https://channels.nixos.org/nixos-25.11/nixexprs.tar.xz";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    nixpkgs_25_11,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [
          (
            final: _prev: {
              pkgs_25_11 = import nixpkgs_25_11 {
                inherit system;
              };
            }
          )
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in {
        formatter = pkgs.alejandra;

        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              pkgs_25_11.mdbook
              mdbook-admonish
              alejandra
            ];
          };
      }
    );
}
