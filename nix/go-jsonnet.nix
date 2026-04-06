{ self, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      rust2gocli = pkgs.rustPlatform.buildRustPackage rec {
        name = "rust2go-cli";
        src = pkgs.fetchCrate {
          pname = name;
          version = "0.4.2";
          hash = "sha256-WT09837Y6lwH6usdoOQ7UTm9HcuHKify/jA8v8R4Fek=";
        };
        cargoHash = "sha256-WP7j+JSByT9fyblcL2saDMmOz2dq0jTSJTjxkTJwn5M=";
      };
    in
    {
      packages = {
        go-jsonnet = pkgs.stdenv.mkDerivation {
          name = "rust2go-vendor";
          src = "${self}/crates/jsonnet-bridge";
          dontUnpack = true;

          nativeBuildInputs = [ pkgs.go ];

          buildPhase = ''
            # We need to compile go in a subdir since go refuses to work in the "system tmp" directory, which is /build in this case
            cp -r $src/* .
            chmod 755 -R .
            # We need to run this manually since the go mod vendor runs before the rust code. Without this we are missing entries in the go.sum
            ${rust2gocli}/bin/rust2go-cli --src src/go.rs --dst go/gen.go
            cd go
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
          outputHash = "sha256-GxgRGTKWO2Bb6cjcFxea4zb5gdEU5V+MKyeKEkFc6UU=";
        };
      };
    };
}
