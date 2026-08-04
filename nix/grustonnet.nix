{
  inputs,
  self,
  lib,
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
      depFile = builtins.fromJSON (builtins.readFile ./../crates/grustonnet-ls-lib/dependencies.json);
      deps = map (dep: {
        inherit (dep) name;
        file = pkgs.fetchurl {
          inherit (dep) url;
          hash = builtins.convertHash {
            inherit (dep) hash;
            toHashFormat = "sri";
            hashAlgo = "sha256";
          };
        };
      }) depFile;

      mkGrustonnet =
        {
          doCheck ? false,
        }:
        naersk'.buildPackage {
          name = "grustonnet-ls";
          src = self;
          inherit doCheck;

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
            ${lib.join "\n" (map (dep: "cp ${dep.file} ./crates/grustonnet-ls-lib/gen/${dep.name}") deps)}
          '';

          nativeBuildInputs = sharedNativeBuildInputs;
          buildInputs = sharedBuildInputs;
          LIBCLANG_PATH = with pkgs; "${llvmPackages.libclang.lib}/lib";
          GODEBUG = "invalidptr=0,cgocheck=0";
          GIT_COMMIT_SHORT = toString (self.shortRev or self.dirtyShortRev or "unknown");
        };

    in
    {
      packages = rec {
        default = grustonnet;
        grustonnet = mkGrustonnet { };
        grustonnet-test = mkGrustonnet { doCheck = true; };
        grustonnetFHSDistribution = pkgs.stdenv.mkDerivation {
          pname = "grustonnet";
          version = "0.0.0";

          buildInputs = [ grustonnet ];
          nativeBuildInputs = [
            pkgs.patchelf
          ];

          dontUnpack = true;

          installPhase = ''
            mkdir -p $out/bin
            cp ${grustonnet}/bin/* $out/bin/
            for bin in $out/bin/*; do
              if [ -f "$bin" ] && [ -x "$bin" ]; then
                chmod +w "$bin"
                patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 "$bin"
              fi
            done
          '';
        };
      };
    };
}
