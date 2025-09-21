{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        jsonnetVersion = "v0.21.0";

        stdlib-content = pkgs.fetchurl {
          url = "https://raw.githubusercontent.com/google/jsonnet/${jsonnetVersion}/doc/_stdlib_gen/stdlib-content.jsonnet";
          hash = "sha256-Xk0V55wYdt9MGNx94DEGS2XW2c9MpYpYl+ly0hi+3vE=";
        };

        stdlib-html = pkgs.fetchurl {
          url = "https://raw.githubusercontent.com/google/jsonnet/${jsonnetVersion}/doc/_stdlib_gen/html.libsonnet";
          hash = "sha256-afCIZAmfLZqyRkxroyHPhJU3ABaEXGQ8xs2TzlrmAAo=";
        };

        naersk' = pkgs.callPackage naersk { };
        modRoot = "./crates/jsonnet-bridge/go";
        goModules = pkgs.stdenv.mkDerivation {
          name = "rust2go-vendor";
          src = ./crates/jsonnet-bridge/go;
          dontUnpack = true;

          nativeBuildInputs = [ pkgs.go ];

          buildPhase = ''
            # Subdir since go refuses to work in the "system tmp" directory, which is /build in this case
            mkdir go && cd go
            cp -r $src/* .
            export GOPATH="$TMPDIR/go-path"
            export GOCACHE="$TMPDIR/go-cache"
            go mod vendor
          '';

          installPhase = ''
            mkdir -p $out
            cp -r vendor $out
          '';

          outputHashMode = "recursive";
          outputHashAlgo = "sha256";
          # Do NOT set to `null` for testing. `go mod vendor` WILL break
          outputHash = "sha256-FFbAMTvtpGkEeUl/TsgfOaYWVkPtHMBuMIzvSQmMhk0=";
        };
        nativeBuildInputs = with pkgs; [
          go

          clang
          pkg-config
        ];

      in
      {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          name = "grustonnet-ls";
          src = ./.;

          # Make sure go can write to the home dir
          preBuild = ''
            export HOME=$TMPDIR
            export GOFLAGS="-mod=vendor"
            mkdir -p ${modRoot}
            ln -s ${goModules}/vendor "${modRoot}/vendor"

            # Download stdlib packages as they are otherwise downloaded by `build.rs`
            mkdir -p ./crates/grustonnet-ls-lib/gen
            cp ${stdlib-content} ./crates/grustonnet-ls-lib/gen/stdlib-content.jsonnet
            cp ${stdlib-html} ./crates/grustonnet-ls-lib/gen/html.libsonnet
          '';

          inherit nativeBuildInputs;
          LIBCLANG_PATH = with pkgs; "${llvmPackages.libclang.lib}/lib";
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            nativeBuildInputs
            ++ [
              cargo
              rustc
              cargo-tarpaulin
              clippy

              go-jsonnet
              rust-analyzer
              bacon
              tracy
            ];
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          LIBCLANG_PATH = with pkgs; "${llvmPackages.libclang.lib}/lib";
        };
      }
    );
}
