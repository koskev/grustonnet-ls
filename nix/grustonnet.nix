{
  inputs,
  self,
  ...
}:
{
  perSystem =
    {
      pkgs,
      self',
      sharedBuildInputs,
      sharedNativeBuildInputs,
      ...
    }:
    let

      naersk' = pkgs.callPackage inputs.naersk { };
      jsonnetVersion = "v0.21.0";
      stdlib-content = pkgs.fetchurl {
        url = "https://raw.githubusercontent.com/google/jsonnet/${jsonnetVersion}/doc/_stdlib_gen/stdlib-content.jsonnet";
        hash = "sha256-Xk0V55wYdt9MGNx94DEGS2XW2c9MpYpYl+ly0hi+3vE=";
      };

      stdlib-html = pkgs.fetchurl {
        url = "https://raw.githubusercontent.com/google/jsonnet/${jsonnetVersion}/doc/_stdlib_gen/html.libsonnet";
        hash = "sha256-afCIZAmfLZqyRkxroyHPhJU3ABaEXGQ8xs2TzlrmAAo=";
      };

      grustonnet = naersk'.buildPackage {
        name = "grustonnet-ls";
        src = self;

        # Make sure go can write to the home dir
        preBuild = ''
          export HOME=$TMPDIR
          export GOFLAGS="-mod=vendor"
          # Copy all the required files to the build directory (=cwd)
          MODROOT="./crates/jsonnet-bridge/go"
          mkdir -p $MODROOT
          ln -s ${self'.packages.go-jsonnet}/vendor "$MODROOT/vendor"

          # Download stdlib packages as they are otherwise downloaded by `build.rs`
          mkdir -p ./crates/grustonnet-ls-lib/gen
          cp ${stdlib-content} ./crates/grustonnet-ls-lib/gen/stdlib-content.jsonnet
          cp ${stdlib-html} ./crates/grustonnet-ls-lib/gen/html.libsonnet
        '';

        nativeBuildInputs = sharedNativeBuildInputs;
        buildInputs = sharedBuildInputs;
        LIBCLANG_PATH = with pkgs; "${llvmPackages.libclang.lib}/lib";
        GODEBUG = "invalidptr=0,cgocheck=0";
      };

    in
    {
      packages = {
        default = grustonnet;
        inherit grustonnet;
      };
    };
}
