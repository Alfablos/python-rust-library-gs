# Rust library for Python example

This *minimal* project illustrates how Rust can be used to write libraries that are then exposed to a Python runtime and imported.

The repo is made up of 2 branches:
* [**master**](https://github.com/Alfablos/python-rust-library-gs/tree/master): the simplest example of how to create a minimal rust library for python

* [**stream-from-file**](https://github.com/Alfablos/python-rust-library-gs/tree/stream-from-file): a more complex example that shows how to asynchronously and lazily read a CSV file using `polars` and stream results to the python runtime that awaits. The `Source` trait is an abstraction, so other sources can be supported.


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
