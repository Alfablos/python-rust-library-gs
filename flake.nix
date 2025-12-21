{
  description = "MIMIC-IV Rust Bindings";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    nix-shells.url = "path:/home/antonio/Documents/nix-shells";
    nix-shells.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      nix-shells,
      ...
    }:
    let
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs
          [
            "x86_64-linux"
            "aarch64-linux"
          ]
          (
            system:
            f (
              import nixpkgs {
                inherit system;
                config.allowUnfree = true;
                overlays = [ ];
              }
            )
          );
      pythonForPkgs = pkgs: pkgs.python3.withPackages (pyPkgs: with pyPkgs; [ ]);
    in
    {
      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.stdenv.hostPlatform.system}.main}/bin/use-python-rust-lib-gs";
        };
      });
      packages = forAllSystems (pkgs: {
        default = self.packages.${pkgs.stdenv.hostPlatform.system}.python-rust-lib-gs;
        main = pkgs.stdenv.mkDerivation {
          name = "use-python-rust-lib-gs";
          propagatedBuildInputs = [
            ((pythonForPkgs pkgs).withPackages (
              pyPkgs: with pyPkgs; [ self.packages.${pkgs.stdenv.hostPlatform.system}.python-rust-lib-gs ]
            ))
          ];
          dontUnpack = true;
          installPhase = "install -Dm755 ${./main.py} $out/bin/use-python-rust-lib-gs";
        };
        python-rust-lib-gs =
          with pkgs;
          with rustPlatform;

          let
            version = (lib.trivial.importTOML ./Cargo.toml).package.version;
            python = pythonForPkgs pkgs;
          in
          python.pkgs.buildPythonPackage {
            pname = "python-rust-lib-gs";
            inherit version;
            src = ./.;

            cargoDeps = importCargoLock {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = [
              cargoSetupHook
              maturinBuildHook
              python.pkgs.setuptools
            ];

            format = "pyproject";
          };
      });
      devShells = forAllSystems (pkgs: {
        default = nix-shells.devShells.${pkgs.stdenv.hostPlatform.system}.rust-stable {
          version = "1.91.1";
          withPkgs = with pkgs; [
            (pkgs.writeShellScriptBin "rustrover" "${tmux}/bin/tmux new -d 'rust-rover $1'")
            maturin
            (pythonForPkgs pkgs)
          ];
          postHook = '''';
        };
      });
    };
}
