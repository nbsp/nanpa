{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;

          nativeBuildInputs = with pkgs; [ scdoc installShellFiles ];
          override.postBuild = ''
            scdoc < ${ ./doc/nanpa.1.scd } > nanpa.1
            scdoc < ${ ./doc/nanparc.5.scd } > nanparc.5
            scdoc < ${ ./doc/nanpa-changeset.5.scd } > nanpa-changeset.5
          '';
          postInstall = ''
            installManPage nanpa.1 nanparc.5 nanpa-changeset.5
          '';
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo rust-analyzer clippy rustfmt scdoc ];
        };
      }
    );
}
