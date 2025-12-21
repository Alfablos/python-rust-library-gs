# Rust library for Python example

This project illustrates how Rust can be used to write libraries that are then exposed to a Python runtime and imported.

## Build

```shell
nix build
```

Alternatively, install `maturin` and run `maturin build --release`. You'll find `$CARGO_TARGET_DIR/wheels/python_rust_lib_gs-${version}-cp31*-abi3-linux_x86_64.whl`.


## Usage

devShell example:

```nix
{
  description = "library client, devShell example";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    rust-python-lib.url = "github:Alfablos/rust-python-library-gs";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-python-lib,
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
      pythonForPkgs =
        pkgs:
        pkgs.python3.withPackages (
          pyPkgs: with pyPkgs; [
            rust-python-lib.outputs.packages.${pkgs.stdenv.hostPlatform.system}.python-rust-lib-gs
            # other useful packages
          ]
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [ (pythonForPkgs pkgs) ];
        };
      });
    };
}

```


test with:

```shell
nix develop

# in the new shell...
python -c 'import python_rust_lib_gs as rpl; print(rpl.FederatedStreamer().message)'
```
"Hey you!" should appear.
