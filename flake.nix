{
  description = "Minimal flake environment";

  inputs = {
    systems.url = "github:nix-systems/default";
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    pre-commit-hooks-nix.url = "github:cachix/pre-commit-hooks.nix";
    rust-overlay = { url = "github:oxalica/rust-overlay"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; }
    {
      systems = import inputs.systems;
      imports = [
        inputs.pre-commit-hooks-nix.flakeModule
      ];
      perSystem =
        { config
          # , self'
          # , inputs'
        , pkgs
        , system
        , lib
        , ...
        }: {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.rust-overlay.overlays.default
              (_self: super: { rustc = super.rust-bin.stable.latest.default; })
            ];
          };

          pre-commit.settings = {
            hooks = {
              deadnix.enable = true;
              nixpkgs-fmt.enable = true;
              statix.enable = true;

              rustfmt = {
                enable = true;
                entry = lib.mkDefault "${pkgs.rustc}/bin/cargo-fmt  -- --color always";
              };
            };
          };

          packages.rustc = pkgs.rustc;

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              bacon
              # openssl.dev
              pkg-config
              stdenv.cc
              rustc
            ];

            shellHook = ''
              ${config.pre-commit.installationScript}
            '';
          };

          formatter = pkgs.nixpkgs-fmt;
        };
      flake = { };
    };
}
