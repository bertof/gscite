{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils = { url = "github:numtide/flake-utils"; };
    rust-overlay = { url = "github:oxalica/rust-overlay"; inputs.nixpkgs.follows = "nixpkgs"; inputs.flake-utils.follows = "flake-utils"; };
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, pre-commit-hooks }:

    let
      overlays = [
        rust-overlay.overlays.default # Rust overlay lib
        (_self: super: { rustc = super.rust-bin.stable.latest.default; }) # Rust overlay
      ];

      multiPlatform = flake-utils.lib.eachDefaultSystem (system:
        let
          # Packages
          pkgs = import nixpkgs { inherit system overlays; };
        in
        with pkgs.lib;
        rec {

          checks = {
            pre-commit-check = pre-commit-hooks.lib.${system}.run {
              src = ./.;
              hooks = {
                nixpkgs-fmt = {
                  enable = true;
                  excludes = [ "Cargo.nix" ];
                };
                nix-linter = {
                  enable = true;
                  excludes = [ "Cargo.nix" ];
                };
                clippy =
                  let
                    wrapper = pkgs.symlinkJoin {
                      name = "clippy-wrapped";
                      paths = [ pkgs.rustc ];
                      nativeBuildInputs = [ pkgs.makeWrapper ];
                      postBuild = ''
                        wrapProgram $out/bin/cargo-clippy \
                          --prefix PATH : ${lib.makeBinPath [ pkgs.rustc ]}
                      '';
                    };
                  in
                  {
                    name = "clippy";
                    description = "Lint Rust code.";
                    entry = "${wrapper}/bin/cargo-clippy clippy";
                    files = "\\.(rs|toml)$";
                    pass_filenames = false;
                  };
                rustfmt =
                  let
                    wrapper = pkgs.symlinkJoin {
                      name = "rustfmt-wrapped";
                      paths = [ pkgs.rustc ];
                      nativeBuildInputs = [ pkgs.makeWrapper ];
                      postBuild = ''
                        wrapProgram $out/bin/cargo-fmt \
                          --prefix PATH : ${lib.makeBinPath [ pkgs.rustc ]}
                      '';
                    };
                  in
                  {
                    name = "rustfmt";
                    description = "Format Rust code.";
                    entry = "${wrapper}/bin/cargo-fmt fmt -- --check --color always";
                    files = "\\.(rs|toml)$";
                    pass_filenames = false;
                  };
              };
            };
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              stdenv.cc.cc.lib
              # rustEnvironment
              # cargo
              rustc
              pkg-config
              openssl.dev

              bacon
              cargo-watch
              cargo-outdated
              clippy
              crate2nix
              rustfmt

            ];

            shellHook = ''
              ${self.checks.${system}.pre-commit-check.shellHook}
            '';
          };
        });

    in
    builtins.foldl' nixpkgs.lib.recursiveUpdate { } [
      multiPlatform
    ];

}
